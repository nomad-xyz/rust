use ethers::prelude::H256;
use prometheus::{Histogram, IntGauge};
use std::{collections::HashMap, time::Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::{debug, info_span, trace, Instrument};

use nomad_ethereum::bindings::replica::UpdateFilter as RelayFilter;

use agent_utils::{
    send_unrecoverable, unwrap_channel_item_unrecoverable, unwrap_pipe_item_unrecoverable,
    ProcessStep, Restartable,
};

use crate::{
    annotate::WithMeta, steps::combine::CombineChannels, RelayFaucet, RelaySink, UpdatePipe,
};

#[derive(Debug)]
pub(crate) struct UpdateWaitMetrics {
    // maps replica network to timing histogram
    pub(crate) times: HashMap<String, Histogram>,
    pub(crate) unrelayed: HashMap<String, IntGauge>,
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
    fn incr_update_gauges(&mut self) {
        self.metrics
            .unrelayed
            .values_mut()
            .for_each(|gauge| gauge.inc());
    }

    fn decr_update_gauge(&self, destination: &str) {
        self.metrics
            .unrelayed
            .get(destination)
            .expect("missing gauge")
            .dec();
    }

    fn start_update_timer(&mut self, root: H256) {
        let now = std::time::Instant::now();
        self.updates.insert(root, now);
        self.incr_update_gauges();
        debug!(
            root = ?root,
            pending_relays = self.pending_relays(),
            "Received new update"
        );
    }

    fn finish_relay_timers(&mut self, root: H256) {
        // mem optimization: remove the hashmap as we no longer need to store
        // any future relay times for this root
        if let Some(mut relays) = self.relays.remove(&root) {
            // times
            relays.drain().for_each(|(destination, relay)| {
                self.record(&destination, relay, *self.updates.get(&root).unwrap());
            })
        }
    }

    fn pending_relays(&self) -> i64 {
        self.metrics
            .unrelayed
            .values()
            .map(|gauge| gauge.get())
            .sum()
    }

    fn relays_tracked(&self) -> usize {
        self.relays.values().map(|v| v.len()).sum()
    }

    fn handle_relay(&mut self, replica_network: &str, root: H256) {
        let now = std::time::Instant::now();

        // mem optimization: don't need to store the relay time
        // if we observe immediately
        if let Some(update_time) = self.updates.get(&root) {
            trace!("Relay for already-seen update");
            self.record(replica_network, now, *update_time);
        } else {
            trace!("Starting timer for relay");
            self.relays
                .entry(root)
                .or_default()
                .insert(replica_network.to_owned(), now);
        }
        debug!(
            replica_network = replica_network,
            root = ?root,
            pending_relays = self.pending_relays(),
            "Handled relay"
        );
    }

    fn handle_update(&mut self, root: H256) {
        self.start_update_timer(root);
        self.finish_relay_timers(root);
    }

    fn record(&self, destination: &str, relay: Instant, update: Instant) {
        let v = relay.saturating_duration_since(update).as_secs_f64();
        trace!(
            elapsed = v,
            destination = destination,
            "Recording relay duration"
        );
        self.metrics
            .times
            .get(destination)
            .expect("missing metric")
            .observe(v);
        self.decr_update_gauge(destination);
    }
}

impl ProcessStep for UpdateWait {
    fn spawn(mut self) -> Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!("UpdateWait", home_network = self.network.as_str());
        tokio::spawn(
            async move {
                trace!(
                    pending_relays = self.pending_relays(),
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
                            trace!(root = ?root, "update pipe item unwrapped");
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
