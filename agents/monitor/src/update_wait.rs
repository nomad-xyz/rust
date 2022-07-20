use ethers::prelude::H256;
use prometheus::Histogram;
use std::{collections::HashMap, time::Instant};
use tokio::sync::mpsc::unbounded_channel;
use tracing::{info_span, Instrument};

use nomad_ethereum::bindings::replica::UpdateFilter as RelayFilter;

use crate::{
    annotate::WithMeta, bail_task_if, utils::SelectChannels, ProcessStep, RelayFaucet, RelaySink,
    Restartable, StepHandle, UpdateFaucet, UpdateSink,
};

#[derive(Debug)]
pub(crate) struct UpdateWaitMetrics {
    pub(crate) times: Histogram,
}

#[derive(Debug)]
pub(crate) struct UpdateWait {
    incoming_update: UpdateFaucet,
    incoming_relays: <SelectChannels<WithMeta<RelayFilter>> as ProcessStep>::Output,

    network: String,
    metrics: UpdateWaitMetrics,

    updates: HashMap<H256, Instant>,
    relays: HashMap<H256, HashMap<String, Instant>>,

    outgoing_update: UpdateSink,
    outgoing_relays: HashMap<String, RelaySink>,
}

impl UpdateWait {
    pub(crate) fn new(
        incoming_update: UpdateFaucet,
        incoming_relays: HashMap<String, RelayFaucet>,
        network: String,
        metrics: UpdateWaitMetrics,
        outgoing_update: UpdateSink,
        outgoing_relays: HashMap<String, RelaySink>,
    ) -> Self {
        let (tx, rx) = unbounded_channel();

        SelectChannels::new(incoming_relays, tx).spawn();

        Self {
            incoming_update,
            incoming_relays: rx,
            network,
            metrics,
            updates: Default::default(),
            relays: Default::default(),
            outgoing_update,
            outgoing_relays,
        }
    }
}

impl std::fmt::Display for UpdateWait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UpdateToRelay - {}", self.network)
    }
}

pub(crate) type UpdateWaitTask = Restartable<UpdateWait>;
pub(crate) type UpdateWaitHandle = StepHandle<UpdateWait>;

#[derive(Debug)]
pub struct UpdateWaitOutput {
    pub(crate) updates: UpdateFaucet,
    pub(crate) relays: HashMap<String, RelayFaucet>,
}

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
    type Output = UpdateWaitOutput;

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

                        update_opt = self.incoming_update.recv() => {
                            bail_task_if!(
                                update_opt.is_none(),
                                self,
                                "Inbound updates broke",
                            );
                            let update = update_opt.unwrap();
                            let root: H256 = update.log.new_root.into();
                            bail_task_if!{
                                // send onwards
                                self.outgoing_update.send(update).is_err(),
                                self,
                                "Outbound updates broke",
                            };

                            self.handle_update(root);
                        }
                        relay_opt = self.incoming_relays.recv() => {
                            bail_task_if!(
                                relay_opt.is_none(),
                                self,
                                format!("Inbound relays broke"),
                            );
                            let (net, relay) = relay_opt.unwrap();
                            let root: H256 = relay.log.new_root.into();

                            bail_task_if!(
                                // send onward
                                self.outgoing_relays
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
