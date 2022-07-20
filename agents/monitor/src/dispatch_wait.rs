use nomad_ethereum::bindings::home::{DispatchFilter, UpdateFilter};
use prometheus::Histogram;
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tracing::{info_span, Instrument};

use crate::{annotate::WithMeta, bail_task_if, ProcessStep, Restartable, StepHandle};

#[derive(Debug)]
pub(crate) struct DispatchWaitMetrics {
    pub(crate) timer: Histogram,
    pub(crate) blocks: Histogram,
}

#[derive(Debug)]
pub(crate) struct DispatchWait {
    incoming_dispatch: UnboundedReceiver<WithMeta<DispatchFilter>>,
    incoming_update: UnboundedReceiver<WithMeta<UpdateFilter>>,
    network: String,
    emitter: String,
    metrics: DispatchWaitMetrics,
    outgoing_update: UnboundedSender<WithMeta<UpdateFilter>>,
    outgoing_dispatch: UnboundedSender<WithMeta<DispatchFilter>>,
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
        incoming_dispatch: UnboundedReceiver<WithMeta<DispatchFilter>>,
        incoming_update: UnboundedReceiver<WithMeta<UpdateFilter>>,
        network: String,
        emitter: String,
        metrics: DispatchWaitMetrics,
        outgoing_update: UnboundedSender<WithMeta<UpdateFilter>>,
        outgoing_dispatch: UnboundedSender<WithMeta<DispatchFilter>>,
    ) -> Self {
        Self {
            incoming_dispatch,
            incoming_update,
            network,
            emitter,
            metrics,
            outgoing_update,
            outgoing_dispatch,
        }
    }
}

pub(crate) type DispatchWaitTask = Restartable<DispatchWait>;
pub(crate) type DispatchWaitHandle = StepHandle<DispatchWait>;

#[derive(Debug)]
pub struct DispatchWaitOutput {
    pub(crate) dispatches: UnboundedReceiver<WithMeta<DispatchFilter>>,
    pub(crate) updates: UnboundedReceiver<WithMeta<UpdateFilter>>,
}

impl ProcessStep for DispatchWait {
    type Output = DispatchWaitOutput;

    fn spawn(mut self) -> DispatchWaitTask
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!(
            "DispatchWait",
            network = self.network.as_str(),
            emitter = self.emitter.as_str(),
        );

        let mut timers = vec![];
        let mut blocks = vec![];

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

                        dispatch_next = self.incoming_dispatch.recv() => {
                            bail_task_if!(
                                dispatch_next.is_none(),
                                self,
                                "inbound dispatch broke"
                            );
                            let dispatch = dispatch_next.expect("checked in block");
                            let block_number = dispatch.meta.block_number;
                            bail_task_if!(
                                self.outgoing_dispatch.send(dispatch).is_err(),
                                self,
                                "outbound dispatch broke"
                            );
                            timers.push(self.metrics.timer.start_timer());
                            blocks.push(block_number);
                        }
                        update_opt = self.incoming_update.recv() => {
                            bail_task_if!(
                                update_opt.is_none(),
                                self,
                                "inbound update broke"
                            );
                            let update = update_opt.expect("checked in block");
                            let block_number = update.meta.block_number;

                            bail_task_if!(
                                self.outgoing_update.send(update).is_err(),
                                self,
                                "outbound update broke"
                            );
                            // drain the entire vec
                            timers.drain(0..).for_each(|timer| timer.observe_duration());
                            blocks.drain(0..).for_each(|dispatch_height| {
                                let diff = block_number.saturating_sub(dispatch_height);
                                self.metrics.blocks.observe(diff.as_u64() as f64);
                            });
                        }
                    }
                }
            }
            .instrument(span),
        )
    }
}
