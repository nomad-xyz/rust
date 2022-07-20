use annotate::WithMeta;
use nomad_ethereum::bindings::{
    home::{DispatchFilter, UpdateFilter},
    replica::{ProcessFilter, UpdateFilter as RelayFilter},
};
use std::{collections::HashMap, panic, sync::Arc};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
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

pub(crate) type Faucet<T> = UnboundedReceiver<WithMeta<T>>;
pub(crate) type Sink<T> = UnboundedSender<WithMeta<T>>;

pub(crate) type DispatchFaucet = Faucet<DispatchFilter>;
pub(crate) type UpdateFaucet = Faucet<UpdateFilter>;
pub(crate) type RelayFaucet = Faucet<RelayFilter>;
pub(crate) type ProcessFaucet = Faucet<ProcessFilter>;
pub(crate) type DispatchSink = Sink<DispatchFilter>;
pub(crate) type UpdateSink = Sink<UpdateFilter>;
pub(crate) type RelaySink = Sink<RelayFilter>;
pub(crate) type ProcessSink = Sink<ProcessFilter>;

pub(crate) type NetworkMap<'a, T> = HashMap<&'a str, T>;
pub(crate) type HomeReplicaMap<'a, T> = HashMap<&'a str, HashMap<&'a str, T>>;

pub(crate) struct Faucets<'a> {
    pub(crate) dispatches: NetworkMap<'a, DispatchFaucet>,
    pub(crate) updates: NetworkMap<'a, UpdateFaucet>,
    pub(crate) relays: HomeReplicaMap<'a, RelayFaucet>,
    pub(crate) processes: HomeReplicaMap<'a, ProcessFaucet>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init::init_tracing();
    {
        let monitor = info_span!("MonitorBootup").in_scope(|| {
            let monitor = init::monitor()?;
            tracing::info!("setup complete!");
            Ok::<_, eyre::Report>(monitor)
        })?;

        let _http = monitor.run_http_server();

        let mut faucets = monitor.producers();

        monitor.run_betweens(&mut faucets);
        monitor.run_dispatch_to_update(&mut faucets);
        monitor.run_update_to_relay(&mut faucets);

        tracing::info!("counters started");

        // just run forever
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10000)).await
        }
    }
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
