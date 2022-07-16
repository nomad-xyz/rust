use tracing::info_span;

mod between;

mod init;

#[tokio::main]
async fn main() {
    init::init_tracing();

    {
        let span = info_span!("MonitorBootup");
        let _span = span.enter();
        let config = init::config();
        let providers = init::init_providers(&config);

        tracing::info!("setup complete!")
    }
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
