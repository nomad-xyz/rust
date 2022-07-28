use ethers::prelude::H256;
use prometheus::Histogram;
use std::{collections::HashMap, time::Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::{debug, info_span, trace, Instrument};

use nomad_ethereum::bindings::replica::UpdateFilter as RelayFilter;

use crate::{
    annotate::WithMeta, pipe::UpdatePipe, send_unrecoverable, steps::combine::CombineChannels,
    unwrap_channel_item_unrecoverable, unwrap_pipe_item_unrecoverable, ProcessStep, RelayFaucet,
    RelaySink,
};

#[derive(Debug)]
pub(crate) struct UpdateWaitMetrics {
    // maps replica network to timing histogram
    pub(crate) times: HashMap<String, Histogram>,
}

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
pub(crate) struct UpdateWait {
    update_pipe: UpdatePipe,

    network: String,
    metrics: UpdateWaitMetrics,

    updates: HashMap<H256, Instant>,
    // maps root -> replica -> timer
    relays: HashMap<H256, HashMap<String, Instant>>,

    relay_faucets: UnboundedReceiver<(String, WithMeta<RelayFilter>)>,
    relay_sinks: HashMap<String, RelaySink>,
}

impl UpdateWait {
    pub(crate) fn new(
        update_pipe: UpdatePipe,

        network: impl AsRef<str>,
        metrics: UpdateWaitMetrics,

        relay_sinks: HashMap<String, RelaySink>,
        relay_faucets: HashMap<String, RelayFaucet>,
    ) -> Self {
        let (tx, rx) = unbounded_channel();

        CombineChannels::new(relay_faucets, tx).run_until_panic();

        Self {
            update_pipe,
            network: network.as_ref().to_owned(),
            metrics,
            updates: Default::default(),
            relays: Default::default(),
            relay_faucets: rx,
            relay_sinks,
        }
    }
}

impl std::fmt::Display for UpdateWait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UpdateToRelay - updates from {}", self.network)
    }
}

impl UpdateWait {
    fn relays_tracked(&self) -> usize {
        self.relays.values().map(|v| v.len()).sum()
    }

    fn handle_relay(&mut self, replica_network: &str, root: H256) {
        let now = std::time::Instant::now();
        debug!(
            replica_network = replica_network,
            root = ?root,
            "Handling relay"
        );
        // mem optimization: don't need to store the relay time
        // if we observe immediately
        if let Some(update_time) = self.updates.get(&root) {
            self.record(replica_network, now, *update_time);
        } else {
            trace!("Starting timer for relay");
            self.relays
                .entry(root)
                .or_default()
                .insert(replica_network.to_owned(), now);
        }
    }

    fn handle_update(&mut self, root: H256) {
        let now = std::time::Instant::now();
        self.updates.insert(root, now);
        debug!(root = ?root, "Starting timers for update");

        // mem optimization: remove the hashmap as we no longer need to store
        // any future relay times for this root
        if let Some(mut relays) = self.relays.remove(&root) {
            // times
            relays.drain().for_each(|(replica_network, relay)| {
                self.record(&replica_network, relay, now);
            })
        }
    }

    fn record(&self, replica_network: &str, relay: Instant, update: Instant) {
        let v = relay.saturating_duration_since(update).as_secs_f64();
        trace!(
            elapsed = v,
            replica_network = replica_network,
            "Recording relay duration"
        );
        self.metrics
            .times
            .get(replica_network)
            .expect("missing metric")
            .observe(v)
    }
}

impl ProcessStep for UpdateWait {
    fn spawn(mut self) -> crate::Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!("UpdateWait", home_network = self.network.as_str());
        tokio::spawn(
            async move {
                trace!(
                    updates_tracked = self.updates.len(),
                    relays_tracked = self.relays_tracked(),
                    "Top of UpdateWait::spawn() loop"
                );
                loop {
                    tokio::select! {
                        // how this works:
                        // For each update, we insert it into the updates map
                        //  We then check the relays map and record differences
                        // For each relay, we insert it into the relays map
                        //  We then check the updates map and record

                        biased;

                        update_opt = self.update_pipe.next() => {
                            trace!("got update pipe item");
                            let update = unwrap_pipe_item_unrecoverable!(update_opt, self);
                            let root: H256 = update.log.new_root.into();
                            trace!(root = ?root, "update chapipennel item unwrapped");
                            self.handle_update(root);
                        }
                        relay_opt = self.relay_faucets.recv() => {
                            trace!("got relay channel item");
                            let (replica_network, relay) = unwrap_channel_item_unrecoverable!(relay_opt, self);
                            let root: H256 = relay.log.new_root.into();
                            trace!(
                                root = ?root,
                                replica_network = replica_network.as_str(),
                                "relay channel item unwrapped"
                            );
                            send_unrecoverable!(self.relay_sinks
                                    .get(&replica_network)
                                    .expect("missing outgoing"), relay, self);

                            self.handle_relay(&replica_network, root);
                         }
                    }
                }
            }
            .instrument(span),
        )
    }
}
