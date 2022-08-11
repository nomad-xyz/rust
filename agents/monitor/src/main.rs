use ethers::prelude::{Http, Provider as EthersProvider};
use futures_util::future::select_all;
use std::sync::Arc;
use tracing::info_span;

use nomad_ethereum::bindings::{
    home::{DispatchFilter, UpdateFilter},
    replica::{ProcessFilter, UpdateFilter as RelayFilter},
};

use agent_utils::{init::init_tracing, pipe::Pipe, Faucet, Sink};

use annotate::WithMeta;

pub(crate) mod annotate;
pub(crate) mod domain;
pub(crate) mod faucets;
pub(crate) mod init;
pub(crate) mod metrics;
pub(crate) mod steps;

pub(crate) type Provider = ethers::prelude::TimeLag<EthersProvider<Http>>;
pub(crate) type ArcProvider = Arc<Provider>;
// pub(crate) type ProviderError = ContractError<Provider>;

pub(crate) type DispatchFaucet = Faucet<WithMeta<DispatchFilter>>;
pub(crate) type UpdateFaucet = Faucet<WithMeta<UpdateFilter>>;
pub(crate) type RelayFaucet = Faucet<WithMeta<RelayFilter>>;
pub(crate) type ProcessFaucet = Faucet<WithMeta<ProcessFilter>>;
pub(crate) type DispatchSink = Sink<WithMeta<DispatchFilter>>;
// pub(crate) type UpdateSink = Sink<WithMeta<UpdateFilter>>;
pub(crate) type RelaySink = Sink<WithMeta<RelayFilter>>;
pub(crate) type ProcessSink = Sink<WithMeta<ProcessFilter>>;

pub(crate) type DispatchPipe = Pipe<WithMeta<DispatchFilter>>;
pub(crate) type UpdatePipe = Pipe<WithMeta<UpdateFilter>>;
pub(crate) type RelayPipe = Pipe<WithMeta<RelayFilter>>;
pub(crate) type ProcessPipe = Pipe<WithMeta<ProcessFilter>>;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init_tracing();
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
        monitor.run_e2e(&mut faucets);

        // sink em
        let tasks = monitor.run_terminals(faucets);

        tracing::info!("tasks started");

        // run until there's a failure of a terminal
        // this would imply there is a series of upstream channel failures
        let (_, _, _) = select_all(tasks).await;
    }

    Ok(())
}
