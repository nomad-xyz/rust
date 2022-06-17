use crate::NomadDB;
use color_eyre::Result;
use nomad_core::{
    NomadEvent, PersistedTransaction, TxDispatchKind, TxOutcome,
};
use std::time::Duration;

const TX_STATUS_POLL_MS: u64 = 100;

/// Transaction poller for submitting PersistentTransaction
#[derive(Debug, Clone)]
pub struct TxPoller {
    db: NomadDB,
}

impl TxPoller {
    /// Create a new TxPoller with a DB ref
    pub fn new(db: NomadDB) -> Self {
        Self { db }
    }

    /// Run the polling loop to send off new transactions
    pub async fn run(&self) -> Result<()> {
        loop {
            tokio::time::sleep(Duration::from_millis(TX_STATUS_POLL_MS)).await;

            let iter = self.db.persisted_transaction_iterator();
            for tx in iter {
                match tx.confirm_event {
                    NomadEvent::Dummy2 => {
                        // send this off
                    }
                    _ => continue,
                }
            }
        }
    }
}

impl std::fmt::Display for TxPoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
