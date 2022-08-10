use ethers::prelude::{H256, U64};
use prometheus::{Histogram, HistogramTimer, IntGauge};
use tokio::select;
use tracing::{debug, info_span, trace, Instrument};

use agent_utils::{unwrap_pipe_item_unrecoverable, ProcessStep, Restartable};

use crate::{DispatchPipe, UpdatePipe};

#[derive(Debug)]
pub(crate) struct DispatchWaitMetrics {
    pub(crate) timer: Histogram,
    pub(crate) blocks: Histogram,
    pub(crate) in_queue: IntGauge,
}

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
pub(crate) struct DispatchWait {
    dispatch_pipe: DispatchPipe,
    update_pipe: UpdatePipe,

    network: String,
    emitter: String,

    metrics: DispatchWaitMetrics,

    timers: Vec<HistogramTimer>,
    blocks: Vec<U64>,
}

impl std::fmt::Display for DispatchWait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DispatchToUpdate latency for {}'s home @ {}",
            self.network, self.emitter,
        )
    }
}

impl DispatchWait {
    pub(crate) fn new(
        dispatch_pipe: DispatchPipe,
        update_pipe: UpdatePipe,
        network: impl AsRef<str>,
        emitter: impl AsRef<str>,
        metrics: DispatchWaitMetrics,
    ) -> Self {
        Self {
            dispatch_pipe,
            update_pipe,
            network: network.as_ref().to_owned(),
            emitter: emitter.as_ref().to_owned(),
            metrics,
            timers: vec![],
            blocks: vec![],
        }
    }

    fn handle_dispatch(&mut self, block_number: U64) {
        self.timers.push(self.metrics.timer.start_timer());
        self.blocks.push(block_number);
        self.metrics.in_queue.set(self.timers.len() as i64);
        debug!(event = "dispatch", "Starting timer for dispatch event",);
    }

    fn handle_update(&mut self, block_number: U64) {
        if !self.timers.is_empty() {
            debug!(count = self.timers.len(), "Ending dispatch timers")
        }

        // drain the entire vec
        self.timers.drain(0..).for_each(|timer| {
            let elapsed = timer.stop_and_record();
            trace!(elapsed = elapsed, "ending dispatch timer");
        });
        self.blocks.drain(0..).for_each(|dispatch_height| {
            let diff = block_number.saturating_sub(dispatch_height);
            self.metrics.blocks.observe(diff.as_u64() as f64);
            trace!(elapsed = %diff, "ending dispatch block count");
        });
        self.metrics.in_queue.set(0);
    }
}

pub(crate) type DispatchWaitTask = Restartable<DispatchWait>;

impl ProcessStep for DispatchWait {
    fn spawn(mut self) -> DispatchWaitTask
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!(
            "DispatchWait",
            network = self.network.as_str(),
            emitter = self.emitter.as_str(),
        );

        tokio::spawn(
            async move {
                loop {
                    trace!(
                        dispatches_tracked = self.timers.len(),
                        "top of DispatchWait::spawn() loop"
                    );
                    // how this works:
                    // For each dispatch, we mark its block height and start a
                    // timer.
                    // Every time an update comes in, we observe all timers, and
                    // then observe all the interblock periods.
                    //
                    // We always send the event onwards before making local
                    // observations, to ensure that the next step gets it
                    // reasonably early

                    select! {
                        // cause the select block to always poll dispatches
                        // first. i.e. ready dispatches will arrive first
                        biased;

                        dispatch_next = self.dispatch_pipe.next() => {
                            trace!("got dispatch pipe item");
                            let dispatch = unwrap_pipe_item_unrecoverable!(dispatch_next, self);
                            let block_number = dispatch.meta.block_number;
                            trace!(
                                block_number = %block_number,
                                message_hash = ?H256::from(dispatch.log.message_hash),
                                "dispatch channel item unwrapped"
                            );
                            self.handle_dispatch(block_number);
                        }
                        update_next = self.update_pipe.next() => {
                            trace!("got update pipe item");
                            let update = unwrap_pipe_item_unrecoverable!(update_next, self);
                            let block_number = update.meta.block_number;
                            trace!(
                                block_number = %block_number,
                                new_root = ?H256::from(update.log.new_root),
                                "update channel item unwrapped"
                            );
                            self.handle_update(block_number);
                        }
                    }
                }
            }
            .instrument(span),
        )
    }
}
