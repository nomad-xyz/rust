use annotate::Annotated;
use std::sync::Arc;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tracing::{info_span, instrument::Instrumented};

use ethers::prelude::{ContractError, Http, Provider as EthersProvider};

pub(crate) mod annotate;
pub(crate) mod between;
pub(crate) mod domain;
pub(crate) mod init;
pub(crate) mod metrics;

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
    }
    Ok(())
}

pub(crate) struct StepHandle<T> {
    handle: Instrumented<JoinHandle<()>>,
    rx: UnboundedReceiver<Annotated<T>>,
}

pub(crate) trait ProcessStep<T> {
    fn spawn(self) -> StepHandle<T>;
}
