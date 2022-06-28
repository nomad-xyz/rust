use crate::NomadDB;
use color_eyre::Result;
use nomad_core::{NomadTxStatus, PersistedTransaction, TxContractStatus, TxEventStatus};
use std::{sync::Arc, time::Duration};

const TX_STATUS_POLL_MS: u64 = 100;

/// Transaction poller for checking tx status against an indexer or a contract
#[derive(Debug, Clone)]
pub struct TxStatus {
    db: NomadDB,
    indexer: Arc<dyn TxEventStatus>,
    contract: Arc<dyn TxContractStatus>,
}

impl TxStatus {
    /// Create a new TxStatus with a DB, indexer and contract ref
    pub fn new(
        db: NomadDB,
        indexer: Arc<dyn TxEventStatus>,
        contract: Arc<dyn TxContractStatus>,
    ) -> Self {
        Self {
            db,
            indexer,
            contract,
        }
    }

    /// Run the polling loop to check transaction status
    pub async fn run(&self) -> Result<()> {
        loop {
            // TODO:
            tokio::time::sleep(Duration::from_millis(TX_STATUS_POLL_MS)).await;
        }
    }
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
