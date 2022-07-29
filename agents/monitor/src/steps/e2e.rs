use std::collections::HashMap;

use nomad_ethereum::bindings::{home::DispatchFilter, replica::ProcessFilter};
use prometheus::{Histogram, IntGauge};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
    time::Instant,
};

use ethers::prelude::H256;
use tracing::{debug, debug_span, info_span, trace, Instrument};

use crate::{
    annotate::WithMeta, send_unrecoverable, unwrap_channel_item_unrecoverable, DispatchFaucet,
    DispatchSink, ProcessFaucet, ProcessSink, ProcessStep,
};

use super::combine::CombineChannels;

pub(crate) struct E2EMetrics {
    // home to times
    pub(crate) timers: HashMap<String, HashMap<String, Histogram>>,
    // home network to remote network to gauges
    pub(crate) gauges: HashMap<String, HashMap<String, IntGauge>>,
}

#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
pub struct E2ELatency {
    dispatch_faucet: UnboundedReceiver<(String, WithMeta<DispatchFilter>)>,
    process_faucet: UnboundedReceiver<(String, (String, WithMeta<ProcessFilter>))>,

    domain_to_network: HashMap<u32, String>,
    metrics: E2EMetrics,

    // home -> destination -> message hash -> time
    dispatches: HashMap<String, HashMap<String, HashMap<H256, Instant>>>,

    // replica_of -> message hash -> time
    processes: HashMap<String, HashMap<H256, Instant>>,

    dispatch_sinks: HashMap<String, DispatchSink>,
    process_sinks: HashMap<String, HashMap<String, ProcessSink>>,
}

impl std::fmt::Display for E2ELatency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E2E latency for the whole network")
    }
}

impl E2ELatency {
    fn timer(&mut self, home: &String, remote: &String) -> &mut Histogram {
        self.metrics
            .timers
            .get_mut(home)
            .expect("missing network")
            .get_mut(remote)
            .expect("missing histogram")
    }

    fn gauge(&mut self, home: &String, remote: &String) -> &mut IntGauge {
        self.metrics
            .gauges
            .get_mut(home)
            .expect("missing network")
            .get_mut(remote)
            .expect("missing gauge")
    }

    fn inc(&mut self, home: &String, remote: &String) -> i64 {
        let gauge = &mut self.gauge(home, remote);
        gauge.inc();
        gauge.get()
    }

    fn dec(&mut self, home: &String, remote: &String) -> i64 {
        let gauge = &mut self.gauge(home, remote);
        gauge.dec();
        gauge.get()
    }

    fn get_dispatch_timer(
        &mut self,
        home: &String,
        remote: &String,
        message_hash: H256,
    ) -> Option<Instant> {
        self.dispatches
            .get_mut(home)
            .and_then(|inner| inner.get_mut(remote))
            .and_then(|inner| inner.remove(&message_hash))
    }

    fn start_dispatch_timer(&mut self, home: &str, remote: &str, message_hash: H256) {
        let now = Instant::now();
        self.dispatches
            .entry(home.to_string())
            .or_default()
            .entry(remote.to_string())
            .or_default()
            .insert(message_hash, now);
    }

    pub(crate) fn new(
        dispatch_faucets: HashMap<String, DispatchFaucet>,
        process_faucets: HashMap<String, HashMap<String, ProcessFaucet>>,
        domain_to_network: HashMap<u32, String>,
        metrics: E2EMetrics,
        dispatch_sinks: HashMap<String, DispatchSink>,
        process_sinks: HashMap<String, HashMap<String, ProcessSink>>,
    ) -> Self {
        let (process_sink, process_faucet) = unbounded_channel();
        let (dispatch_sink, dispatch_faucet) = unbounded_channel();

        CombineChannels::new(dispatch_faucets, dispatch_sink).run_until_panic();
        CombineChannels::nested(process_faucets, process_sink).run_until_panic();

        Self {
            dispatch_faucet,
            process_faucet,
            domain_to_network,
            metrics,
            dispatches: Default::default(),
            processes: Default::default(),
            dispatch_sinks,
            process_sinks,
        }
    }

    fn record_dispatch(&mut self, network: String, destination: u32, message_hash: H256) {
        if !self.domain_to_network.contains_key(&destination) {
            tracing::trace!("dispatch to un-monitored network");
            return;
        }
        let destination = self
            .domain_to_network
            .get(&destination)
            .expect("checked")
            .clone();

        let _span = debug_span!(
            "record_dispatch",
            network = network.as_str(),
            destination,
            message_hash = ?message_hash,
        )
        .entered();

        debug!("Recording dispatch");
        // if we know of a matching process on the appropriate destination
        // mark it
        // otherwise store in dispatch map
        if self
            .processes
            .get_mut(&network)
            .and_then(|entry| entry.remove(&message_hash))
            .is_some()
        {
            trace!(elapsed = 0.0, "dispatch preceded by process");
            self.timer(&network, &destination).observe(0.0);
        } else {
            self.start_dispatch_timer(&network, &destination, message_hash);

            let unprocessed_dispatches = self.inc(&network, &destination);
            trace!(unprocessed_dispatches, "Started dispatch e2e timer");
        }
    }

    fn record_process(&mut self, network: String, replica_of: String, message_hash: H256) {
        debug_span!(
            "record_process",
            network = network.as_str(),
            replica_of = replica_of.as_str(),
            message_hash = ?message_hash
        );
        let now = Instant::now();

        // if we know of a matching dispatch, mark it and remove from map
        if let Some(dispatch) = self.get_dispatch_timer(&replica_of, &network, message_hash) {
            let time = now.saturating_duration_since(dispatch).as_secs_f64();

            // gauges is keyed by the home network, so we use replica_of here
            let unprocessed_dispatches = self.dec(&replica_of, &network);
            self.timer(&replica_of, &network).observe(time);
            debug!(
                unprocessed_dispatches,
                time, "Recorded process w/ matching dispatch",
            );
        } else {
            debug!("Recording process w/o matching dispatch");
            // record it for later
            self.processes
                .entry(replica_of)
                .or_default()
                .insert(message_hash, now);
        }
    }
}

impl ProcessStep for E2ELatency {
    fn spawn(mut self) -> crate::Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!("E2ELatency");
        tokio::spawn(
            async move {
                loop {
                    tokio::select! {
                        dispatch_opt = self.dispatch_faucet.recv() => {
                            let (network, dispatch) = unwrap_channel_item_unrecoverable!(dispatch_opt, self);
                            let message_hash: H256 = dispatch.log.message_hash.into();
                            let destination: u32  = (dispatch.log.destination_and_nonce >> 32) as u32;

                            let outbound = self.dispatch_sinks.get(&network).expect("missing sink");
                            send_unrecoverable!(outbound, dispatch, self);

                            self.record_dispatch(network,destination, message_hash);
                        }
                        process_opt = self.process_faucet.recv() => {
                            let (network, (replica_of, process)) = unwrap_channel_item_unrecoverable!(process_opt, self);

                            let message_hash: H256 = process.log.message_hash.into();

                            let outbound = self.process_sinks.get(&network).expect("missing network").get(&replica_of).expect("missing sink");

                            send_unrecoverable!(outbound, process, self);

                            self.record_process(network, replica_of, message_hash);
                        }
                    }
                }
            }
            .instrument(span),
        )
    }
}
