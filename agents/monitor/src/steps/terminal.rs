use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug_span, Instrument};

use crate::{ProcessStep, Restartable};

#[derive(Debug)]
#[must_use = "Tasks do nothing unless you call .spawn() or .run_until_panic()"]
/// A process step that just drains its input and drops everything
/// Its [`StepHandle`] will never produce values.
pub(crate) struct Terminal<T>
where
    T: std::fmt::Debug,
{
    pub(crate) rx: UnboundedReceiver<T>,
}

impl<T> Terminal<T>
where
    T: std::fmt::Debug,
{
    pub(crate) fn new(rx: UnboundedReceiver<T>) -> Self {
        Self { rx }
    }
}

impl<T> std::fmt::Display for Terminal<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Terminal")
    }
}

pub(crate) type TerminalHandle<T> = Restartable<Terminal<T>>;

impl<T> ProcessStep for Terminal<T>
where
    T: std::fmt::Debug + Send + Sync + 'static,
{
    fn spawn(mut self) -> TerminalHandle<T> {
        let span = debug_span!("Terminal Handler");
        tokio::spawn(
            async move {
                loop {
                    if self.rx.recv().await.is_none() {
                        tracing::debug!(self = %self, "Upstream broke, shutting down");
                        return (self, eyre::eyre!(""));
                    }
                }
            }
            .instrument(span),
        )
    }
}
