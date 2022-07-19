use nomad_ethereum::bindings::home::{DispatchFilter, UpdateFilter};
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tracing::{info_span, Instrument};

use crate::{annotate::WithMeta, bail_task_if, ProcessStep, Restartable, StepHandle};

#[derive(Debug)]
pub(crate) struct DispatchWait {
    incoming_dispatch: UnboundedReceiver<WithMeta<DispatchFilter>>,
    incoming_update: UnboundedReceiver<WithMeta<UpdateFilter>>,
    network: String,
    emitter: String,
    outgoing_update: UnboundedSender<WithMeta<UpdateFilter>>,
}

pub(crate) type DispatchWaitTask = Restartable<DispatchWait>;
pub(crate) type DispatchWaitHandle = StepHandle<DispatchWait, WithMeta<UpdateFilter>>;

impl ProcessStep<WithMeta<UpdateFilter>> for DispatchWait {
    fn spawn(mut self) -> DispatchWaitTask
    where
        Self: 'static + Send + Sync + Sized,
    {
        let span = info_span!(
            "DispatchWait",
            network = self.network.as_str(),
            emitter = self.emitter.as_str(),
        );
        let mut dispatches = vec![];
        // TODO: timers vec here
        tokio::spawn(
            async move {
                loop {
                    select! {
                        dispatch_next = self.incoming_dispatch.recv() => {
                            bail_task_if!(
                                dispatch_next.is_none(),
                                self,
                                "inbound dispatch broke"
                            );
                            // TODO: push timer
                            dispatches.push(dispatch_next.expect("checked in block"));
                        }
                        update_opt = self.incoming_update.recv() => {
                            bail_task_if!(
                                update_opt.is_none(),
                                self,
                                "inbound update broke"
                            );
                            // TODO: close out all timers here
                            bail_task_if!(
                                self.outgoing_update.send(update_opt.expect("checked in block")).is_err(),
                                self,
                                "outbound update broke"
                            );
                        }
                    }
                    todo!()
                }
            }
            .instrument(span),
        )
    }
}
