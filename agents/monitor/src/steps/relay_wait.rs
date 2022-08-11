use std::time::Duration;

use agent_utils::Restartable;
use ethers::prelude::U64;
use prometheus::Histogram;
use tokio::time::Instant;

use tracing::{debug, info_span, trace, Instrument};

use agent_utils::{unwrap_pipe_item_unrecoverable, ProcessStep};

use crate::{ProcessPipe, RelayPipe};

#[derive(Debug)]
pub(crate) struct RelayWaitMetrics {
    pub(crate) timers: Histogram,
    pub(crate) blocks: Histogram,
}

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
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
    fn spawn(mut self) -> Restartable<Self>
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
                    trace!(
                        relay_tracked = !self.relay_block.is_zero(),
                        "top of RelayWait::spawn() loop"
                    );
                    tokio::select! {
                        biased;

                        process_next = self.process_pipe.next() => {
                            let process = unwrap_pipe_item_unrecoverable!(process_next, self);

                            if !self.relay_block.is_zero() {
                                let process_instant = tokio::time::Instant::now();
                                let process_block = process.meta.block_number;

                                let elapsed = process_instant.saturating_duration_since(self.relay_instant).as_secs_f64();
                                let elapsed_blocks = process_block.saturating_sub(self.relay_block).as_u64() as f64;

                                debug!(
                                    elapsed_blocks = elapsed_blocks,
                                    elapsed = elapsed,
                                    "Recording time since relay"
                                );
                                self.metrics.timers.observe(elapsed);
                                self.metrics.blocks.observe(elapsed_blocks);
                            }
                        }
                        relay_next = self.relay_pipe.next() => {
                            let relay = unwrap_pipe_item_unrecoverable!(relay_next, self);
                            debug!(
                                relay_block = %relay.meta.block_number,
                                "Starting relay timers"
                            );
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
