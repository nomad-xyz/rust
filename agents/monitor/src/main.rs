use annotate::WithMeta;
use std::{panic, sync::Arc};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tracing::{debug_span, info_span, instrument::Instrumented, Instrument};

use ethers::prelude::{ContractError, Http, Provider as EthersProvider};

pub(crate) mod annotate;
pub(crate) mod between;
pub(crate) mod domain;
pub(crate) mod init;
pub(crate) mod macros;
pub(crate) mod metrics;
pub(crate) mod producer;

pub(crate) type Provider = ethers::prelude::TimeLag<EthersProvider<Http>>;
pub(crate) type ArcProvider = Arc<Provider>;
pub(crate) type ProviderError = ContractError<Provider>;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init::init_tracing();
    {
        let span = info_span!("MonitorBootup");
        let _span = span.enter();

        let monitor = init::monitor()?;
        tracing::info!("setup complete!");
        let _http = monitor.run_http_server();

        let dispatch_trackers = monitor.run_between_dispatch();

        // should cause it to run until crashes occur
        dispatch_trackers.into_iter().next().unwrap().1.await;
    }
    Ok(())
}

pub type Restartable<Task> = Instrumented<JoinHandle<(Task, eyre::Report)>>;

pub(crate) struct StepHandle<Task, Produces> {
    handle: Restartable<Task>,
    rx: UnboundedReceiver<Produces>,
}

pub(crate) trait ProcessStep<T>
where
    T: 'static + Send + Sync + std::fmt::Debug,
{
    fn spawn(self) -> Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized;

    fn forever(self) -> JoinHandle<()>
    where
        Self: 'static + Send + Sync + Sized,
    {
        tokio::spawn(async move {
            let mut handle = self.spawn();
            loop {
                let result = handle.await;

                let again = match result {
                    Ok((handle, report)) => {
                        tracing::warn!(error = %report, "Restarting task");
                        handle
                    }
                    Err(e) => {
                        tracing::error!(err = %e, "JoinError in forever");
                        panic!("JoinError in forever");
                    }
                };
                handle = again.spawn()
            }
        })
    }
}

/// A process step that just drains its input and drops everything
/// Its [`StepHandle`] will never produce values.
pub(crate) struct Terminal<T>
where
    T: std::fmt::Debug,
{
    rx: UnboundedReceiver<WithMeta<T>>,
}

pub(crate) type TerminalHandle<T> = Restartable<Terminal<T>>;

impl<T> ProcessStep<T> for Terminal<T>
where
    T: 'static + Send + Sync + std::fmt::Debug,
{
    fn spawn(mut self) -> TerminalHandle<T> {
        let span = debug_span!("Terminal Handler");
        tokio::spawn(async move {
            loop {
                if self.rx.recv().await.is_none() {
                    tracing::info!("Upstream broke, shutting down");
                    return (self, eyre::eyre!(""));
                }
            }
        })
        .instrument(span)
    }
}
