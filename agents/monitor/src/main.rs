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
pub(crate) mod domain;
pub(crate) mod faucets;
pub(crate) mod init;
pub(crate) mod macros;
pub(crate) mod metrics;
pub(crate) mod pipe;
pub(crate) mod steps;
pub(crate) mod utils;

pub(crate) type Provider = ethers::prelude::TimeLag<EthersProvider<Http>>;
pub(crate) type ArcProvider = Arc<Provider>;
// pub(crate) type ProviderError = ContractError<Provider>;

pub(crate) type Restartable<Task> = JoinHandle<(Task, eyre::Report)>;

pub(crate) type Faucet<T> = UnboundedReceiver<T>;
pub(crate) type Sink<T> = UnboundedSender<T>;

pub(crate) type DispatchFaucet = Faucet<WithMeta<DispatchFilter>>;
pub(crate) type UpdateFaucet = Faucet<WithMeta<UpdateFilter>>;
pub(crate) type RelayFaucet = Faucet<WithMeta<RelayFilter>>;
pub(crate) type ProcessFaucet = Faucet<WithMeta<ProcessFilter>>;
pub(crate) type DispatchSink = Sink<WithMeta<DispatchFilter>>;
pub(crate) type UpdateSink = Sink<WithMeta<UpdateFilter>>;
pub(crate) type RelaySink = Sink<WithMeta<RelayFilter>>;
pub(crate) type ProcessSink = Sink<WithMeta<ProcessFilter>>;

pub(crate) type NetworkMap<'a, T> = HashMap<&'a str, T>;
pub(crate) type HomeReplicaMap<'a, T> = HashMap<&'a str, HashMap<&'a str, T>>;

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
        monitor.run_relay_to_process(&mut faucets);

        tracing::info!("tasks started");

        // just run forever
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10000)).await
        }
    }
}

pub(crate) trait ProcessStep: std::fmt::Display {
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

pub(crate) struct StepHandle<Task, Output> {
    handle: Restartable<Task>,
    rx: Output,
}
