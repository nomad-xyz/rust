use ethers::prelude::U64;
use prometheus::{Histogram, HistogramTimer};
use tokio::select;
use tracing::{info_span, Instrument};

use crate::{
    pipe::{DispatchPipe, UpdatePipe},
    unwrap_pipe_item, ProcessStep, Restartable,
};

#[derive(Debug)]
pub(crate) struct DispatchWaitMetrics {
    pub(crate) timer: Histogram,
    pub(crate) blocks: Histogram,
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
    }

    fn handle_update(&mut self, block_number: U64) {
        // drain the entire vec
        self.timers
            .drain(0..)
            .for_each(|timer| timer.observe_duration());
        self.blocks.drain(0..).for_each(|dispatch_height| {
            let diff = block_number.saturating_sub(dispatch_height);
            self.metrics.blocks.observe(diff.as_u64() as f64);
        });
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
                            let dispatch = unwrap_pipe_item!(dispatch_next,self);
                            let block_number = dispatch.meta.block_number;
                            self.handle_dispatch(block_number);
                        }
                        update_next = self.update_pipe.next() => {
                            let update = unwrap_pipe_item!(update_next, self);
                            let block_number = update.meta.block_number;
                            self.handle_update(block_number);
                        }
                    }
                }
            }
            .instrument(span),
        )
    }
}
