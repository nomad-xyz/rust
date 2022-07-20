use std::{collections::HashMap, panic, sync::Arc};
use tokio::task::JoinHandle;
use tracing::info_span;

use ethers::prelude::{Http, Provider as EthersProvider};

pub(crate) mod annotate;
pub(crate) mod between;
pub(crate) mod dispatch_wait;
pub(crate) mod domain;
pub(crate) mod init;
pub(crate) mod macros;
pub(crate) mod metrics;
pub(crate) mod producer;
pub(crate) mod terminal;
pub(crate) mod update_wait;
pub(crate) mod utils;

pub(crate) type Provider = ethers::prelude::TimeLag<EthersProvider<Http>>;
pub(crate) type ArcProvider = Arc<Provider>;
// pub(crate) type ProviderError = ContractError<Provider>;

pub(crate) type HomeReplicaMap<'a, T> = HashMap<&'a str, HashMap<&'a str, T>>;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init::init_tracing();
    {
        let span = info_span!("MonitorBootup");
        let _span = span.enter();

        let monitor = init::monitor()?;
        tracing::info!("setup complete!");
        let _http = monitor.run_http_server();

        let dispatch_producer = monitor.run_dispatch_producers();
        let update_producer = monitor.run_update_producers();
        let relay_producers = monitor.run_relay_producers();
        let process_producers = monitor.run_process_producers();

        let (dispatch_handles, dispatch_producer) = utils::split(dispatch_producer);
        let (update_handles, update_producer) = utils::split(update_producer);
        let (relay_handles, relay_producers) = utils::nested_split(relay_producers);
        let (process_handles, process_producers) = utils::nested_split(process_producers);

        let dispatch_counters = monitor.run_between_dispatch(dispatch_producer);
        let update_counters = monitor.run_between_update(update_producer);
        let relay_counters = monitor.run_between_relay(relay_producers);
        let process_counters = monitor.run_between_process(process_producers);

        let (dispatch_count_handles, dispatch_producer) = utils::split(dispatch_counters);
        let (update_count_handles, update_producer) = utils::split(update_counters);
        let (relay_count_handles, relay_producer) = utils::nested_split(relay_counters);
        let (process_count_handles, process_producer) = utils::nested_split(process_counters);

        let d_to_u = monitor.run_dispatch_to_update(dispatch_producer, update_producer);

        let (d_to_u_handles, d_and_u_producers) = utils::split(d_to_u);

        tracing::info!("counters started");

        // should cause it to run until crashes occur
        let _ = update_handles.into_iter().next().unwrap().await;
    }
    Ok(())
}

pub type Restartable<Task> = JoinHandle<(Task, eyre::Report)>;

/// A step handle is the handle to the process, and its outbound channels.
///
/// Task creation reutns a step handle so that we can
/// - track the
pub(crate) struct StepHandle<Task>
where
    Task: ProcessStep,
{
    handle: Restartable<Task>,
    rx: <Task as ProcessStep>::Output,
}

pub(crate) trait ProcessStep: std::fmt::Display {
    type Output: 'static + Send + Sync + std::fmt::Debug;

    fn spawn(self) -> Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized;

    /// Run the task until it panics. Errors result in a task restart with the
    /// same channels. This means that an error causes the task to lose only
    /// the data that is in-scope when it faults.
    fn run_until_panic(self) -> Restartable<Self>
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
                        tracing::error!(err = %e, "JoinError in forever. Internal task panicked");
                        panic!("JoinError in forever. Internal task panicked");
                    }
                };
                handle = again.spawn()
            }
        })
    }
}
