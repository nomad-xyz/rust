use std::collections::HashMap;

use nomad_ethereum::bindings::{home::DispatchFilter, replica::ProcessFilter};
use prometheus::Histogram;
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
    pub(crate) timers: HashMap<String, Histogram>,
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
        let _span = debug_span!(
            "record_dispatch",
            network = network.as_str(),
            destination,
            message_hash = ?message_hash,
            destination_network = self.domain_to_network.get(&destination).unwrap().as_str(),
        )
        .entered();
        debug!("Recording dispatch");
        // ignore unknown destinations
        if let Some(destination) = self.domain_to_network.get(&destination) {
            let now = Instant::now();

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
                self.metrics
                    .timers
                    .get_mut(&network)
                    .unwrap()
                    .observe(0 as f64);
            } else {
                trace!("Starting dispatch e2e timer");
                self.dispatches
                    .entry(network.clone())
                    .or_default()
                    .entry(destination.to_owned())
                    .or_default()
                    .insert(message_hash, now);
            }
        }
    }

    fn record_process(&mut self, network: String, replica_of: String, message_hash: H256) {
        debug_span!(
            "record_process",
            network = network.as_str(),
            replica_of = replica_of.as_str(),
            message_hash = ?message_hash
        );
        tracing::debug!("Recording process");
        let now = Instant::now();

        // if we know of a matching dispatch, mark it and remove from map
        if let Some(dispatch) = self
            .dispatches
            .get_mut(&replica_of)
            .and_then(|inner| inner.get_mut(&network))
            .and_then(|inner| inner.remove(&message_hash))
        {
            trace!("Matching dispatch found.");
            let time = now.saturating_duration_since(dispatch).as_secs_f64();
            self.metrics
                .timers
                .get_mut(&replica_of)
                .unwrap()
                .observe(time);
        } else {
            trace!("No matching dispatch found");
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
