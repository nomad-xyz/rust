use init::Monitor;
use tracing::info_span;

mod between;

mod init;

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
