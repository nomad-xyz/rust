use tracing::info_span;

pub(crate) mod between;
pub(crate) mod domain;
pub(crate) mod init;
pub(crate) mod metrics;

use std::sync::Arc;

use ethers::prelude::{ContractError, Http, Provider as EthersProvider, StreamExt};

use nomad_ethereum::bindings::{
    home::UpdateFilter as HomeUpdateFilter, replica::UpdateFilter as ReplicaUpdateFilter,
};
use prometheus::{HistogramOpts, HistogramVec, IntCounterVec};
use tokio::{
    sync::mpsc::{self},
    task::JoinHandle,
};

pub(crate) type Provider = ethers::prelude::TimeLag<EthersProvider<Http>>;
pub(crate) type ArcProvider = Arc<Provider>;
pub(crate) type ProviderError = ContractError<Provider>;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init::init_tracing();
    {
        let span = info_span!("MonitorBootup");
        let _span = span.enter();

        let _monitor = init::monitor()?;

        tracing::info!("setup complete!")
    }
    Ok(())
}

/// Simple event trait
pub trait NomadEvent: Send + Sync {
    /// Get the timestamp
    fn timestamp(&self) -> u32;

    /// block number
    fn block_number(&self) -> u64;

    /// tx hash
    fn tx_hash(&self) -> ethers::types::H256;
}
