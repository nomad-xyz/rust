use std::time::Duration;

use ethers::prelude::U64;
use prometheus::Histogram;
use tokio::time::Instant;

use tracing::{info_span, Instrument};

use crate::{
    bail_task_if,
    pipe::{ProcessPipe, RelayPipe},
    unwrap_pipe_item, ProcessStep,
};

#[derive(Debug)]
pub(crate) struct RelayWaitMetrics {
    pub(crate) timers: Histogram,
    pub(crate) blocks: Histogram,
}

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .forever()"]
pub(crate) struct RelayWait {
    relay_pipe: RelayPipe,
    process_pipe: ProcessPipe,

    network: String,
    replica_of: String,
    emitter: String,
    metrics: RelayWaitMetrics,

    relay_instant: Instant,
    relay_block: U64,
}

impl RelayWait {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        relay_pipe: RelayPipe,
        process_pipe: ProcessPipe,

        network: String,
        replica_of: String,
        emitter: String,
        metrics: RelayWaitMetrics,
    ) -> Self {
        Self {
            relay_pipe,
            process_pipe,
            network,
            replica_of,
            emitter,
            metrics,
            relay_instant: Instant::now() + Duration::from_secs(86400 * 30 * 12 * 30),
            relay_block: U64::zero(),
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

                        process_next = self.process_pipe.next() => {
                            let process = unwrap_pipe_item!(process_next, self);
                            let process_instant = tokio::time::Instant::now();
                            let process_block = process.meta.block_number;

                            let elapsed_ms = process_instant.saturating_duration_since(self.relay_instant).as_millis() as f64;
                            let elapsed_blocks = process_block.saturating_sub(self.relay_block).as_u64() as f64;

                            self.metrics.timers.observe(elapsed_ms);
                            self.metrics.blocks.observe(elapsed_blocks);
                        }
                        relay_next = self.relay_pipe.next() => {
                            let relay = unwrap_pipe_item!(relay_next, self);
                            self.relay_instant = tokio::time::Instant::now();
                            self.relay_block = relay.meta.block_number;
                        }

                    }
                }
            }
            .instrument(span),
        )
    }
}
