use std::time::Duration;

use ethers::prelude::U64;
use prometheus::Histogram;
use tokio::time::Instant;

use tracing::{info_span, Instrument};

use crate::{bail_task_if, ProcessFaucet, ProcessSink, ProcessStep, RelayFaucet, RelaySink};

#[derive(Debug)]
pub struct RelayWaitMetrics {
    timers: Histogram,
    blocks: Histogram,
}

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .forever()"]
pub(crate) struct RelayWait {
    relay_faucet: RelayFaucet,
    process_faucet: ProcessFaucet,

    network: String,
    replica_of: String,
    emitter: String,
    metrics: RelayWaitMetrics,

    relay_instant: Instant,
    relay_block: U64,

    relay_sink: RelaySink,
    process_sink: ProcessSink,
}

impl RelayWait {
    pub(crate) fn new(
        relay_faucet: RelayFaucet,
        process_faucet: ProcessFaucet,
        network: String,
        replica_of: String,
        emitter: String,
        metrics: RelayWaitMetrics,
        relay_sink: RelaySink,
        process_sink: ProcessSink,
    ) -> Self {
        Self {
            relay_faucet,
            process_faucet,
            network,
            replica_of,
            emitter,
            metrics,
            relay_instant: Instant::now() + Duration::from_secs(86400 * 30 * 12 * 30),
            relay_block: U64::zero(),
            relay_sink,
            process_sink,
        }
    }
}

impl std::fmt::Display for RelayWait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RelayToProcess latency for {}'s replica of {} @ {}",
            self.network, self.replica_of, self.emitter,
        )
    }
}

impl ProcessStep for RelayWait {
    fn spawn(mut self) -> crate::Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!(
            "ProcessWait",
            network = self.network.as_str(),
            emitter = self.emitter.as_str(),
        );
        tokio::spawn(
            async move {
                loop {
                    tokio::select! {
                        biased;

                        process_next = self.process_faucet.recv() => {
                            bail_task_if!(
                                process_next.is_none(),
                                self,
                                "inbound relay broke"
                            );
                            let process = process_next.expect("checked");
                            let process_instant = tokio::time::Instant::now();
                            let process_block = process.meta.block_number;
                            bail_task_if!(
                                self.process_sink.send(process).is_err(),
                                self,
                                "outbound relay broke",
                            );
                            let elapsed_ms = process_instant.saturating_duration_since(self.relay_instant).as_millis() as f64;
                            let elapsed_blocks = process_block.saturating_sub(self.relay_block).as_u64() as f64;

                            self.metrics.timers.observe(elapsed_ms);
                            self.metrics.blocks.observe(elapsed_blocks);
                        }
                        relay_next = self.relay_faucet.recv() => {
                            bail_task_if!(
                                relay_next.is_none(),
                                self,
                                "inbound relay broke"
                            );
                            let relay = relay_next.expect("checked");
                            self.relay_instant = tokio::time::Instant::now();
                            self.relay_block = relay.meta.block_number;
                            bail_task_if!(
                                self.relay_sink.send(relay).is_err(),
                                self,
                                "outbound relay broke",
                            );
                        }

                    }
                }
            }
            .instrument(span),
        )
    }
}
