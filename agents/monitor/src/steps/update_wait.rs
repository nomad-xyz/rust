use ethers::prelude::H256;
use prometheus::Histogram;
use std::{collections::HashMap, time::Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::{info_span, Instrument};

use nomad_ethereum::bindings::replica::UpdateFilter as RelayFilter;

use crate::{
    annotate::WithMeta, bail_task_if, utils::SelectChannels, ProcessStep, RelayFaucet, RelaySink,
    Restartable, UpdateFaucet, UpdateSink,
};

#[derive(Debug)]
pub(crate) struct UpdateWaitMetrics {
    pub(crate) times: Histogram,
}

#[derive(Debug)]
pub(crate) struct UpdateWait {
    update_faucet: UpdateFaucet,
    relay_faucets: UnboundedReceiver<(String, WithMeta<RelayFilter>)>,

    network: String,
    metrics: UpdateWaitMetrics,

    updates: HashMap<H256, Instant>,
    relays: HashMap<H256, HashMap<String, Instant>>,

    update_sink: UpdateSink,
    relay_sinks: HashMap<String, RelaySink>,
}

impl UpdateWait {
    pub(crate) fn new(
        update_faucet: UpdateFaucet,
        relay_faucets: HashMap<String, RelayFaucet>,
        network: impl AsRef<str>,
        metrics: UpdateWaitMetrics,
        update_sink: UpdateSink,
        relay_sinks: HashMap<String, RelaySink>,
    ) -> Self {
        let (tx, rx) = unbounded_channel();

        SelectChannels::new(relay_faucets, tx).spawn();

        Self {
            update_faucet,
            relay_faucets: rx,
            network: network.as_ref().to_owned(),
            metrics,
            updates: Default::default(),
            relays: Default::default(),
            update_sink,
            relay_sinks,
        }
    }
}

impl std::fmt::Display for UpdateWait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UpdateToRelay - {}", self.network)
    }
}

pub(crate) type UpdateWaitTask = Restartable<UpdateWait>;

impl UpdateWait {
    fn handle_relay(&mut self, net: &str, root: H256) {
        let now = std::time::Instant::now();

        // mem optimization: don't need to store the relay time
        // if we observe immediately
        if let Some(update_time) = self.updates.get(&root) {
            self.record(now, *update_time)
        } else {
            self.relays
                .entry(root)
                .or_default()
                .insert(net.to_owned(), now);
        }
    }

    fn handle_update(&mut self, root: H256) {
        let now = std::time::Instant::now();
        self.updates.insert(root, now);

        // mem optimization: remove the hashmap as we no longer need to store
        // any future relay times for this root
        if let Some(mut relays) = self.relays.remove(&root) {
            // times
            relays.drain().for_each(|(_, relay)| {
                self.record(relay, now);
            })
        }
    }

    fn record(&self, relay: Instant, update: Instant) {
        let v = relay.saturating_duration_since(update);
        self.metrics.times.observe(v.as_millis() as f64)
    }
}

impl ProcessStep for UpdateWait {
    fn spawn(mut self) -> crate::Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!("UpdateWait");
        tokio::spawn(
            async move {
                loop {
                    tokio::select! {
                        // how this works:
                        // For each update, we insert it into the updates map
                        //  We then check the relays map and record differences
                        // For each relay, we insert it into the relays map
                        //  We then check the updates map and record

                        biased;

                        update_opt = self.update_faucet.recv() => {
                            bail_task_if!(
                                update_opt.is_none(),
                                self,
                                "Inbound updates broke",
                            );
                            let update = update_opt.unwrap();
                            let root: H256 = update.log.new_root.into();
                            bail_task_if!{
                                // send onwards
                                self.update_sink.send(update).is_err(),
                                self,
                                "Outbound updates broke",
                            };

                            self.handle_update(root);
                        }
                        relay_opt = self.relay_faucets.recv() => {
                            bail_task_if!(
                                relay_opt.is_none(),
                                self,
                                format!("Inbound relays broke"),
                            );
                            let (net, relay) = relay_opt.unwrap();
                            let root: H256 = relay.log.new_root.into();

                            bail_task_if!(
                                // send onward
                                self.relay_sinks
                                    .get(&net)
                                    .expect("missing outgoing")
                                    .send(relay)
                                    .is_err(),
                                self,
                                format!("outgoing relay for {} broke", &net)
                            );

                            self.handle_relay(&net, root);
                         }
                    }
                }
            }
            .instrument(span),
        )
    }
}
