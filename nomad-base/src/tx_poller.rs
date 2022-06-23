use crate::NomadDB;
use color_eyre::Result;
use nomad_core::{NomadEvent, PersistedTransaction, TxForwarder};
use std::{sync::Arc, time::Duration};

const TX_STATUS_POLL_MS: u64 = 100;

/// Transaction poller for submitting PersistentTransaction
#[derive(Debug, Clone)]
pub struct TxPoller {
    db: NomadDB,
    contract: Arc<dyn TxForwarder>,
}

impl TxPoller {
    /// Create a new TxPoller with a DB ref
    pub fn new(db: NomadDB, contract: Arc<dyn TxForwarder>) -> Self {
        Self { db, contract }
    }

    /// Return the next tx that needs sending
    fn next_transaction(&self) -> Option<PersistedTransaction> {
        let mut iter = self.db.persisted_transaction_iterator();
        iter.find(|tx| tx.confirm_event == NomadEvent::Dummy2)
    }

    /// Run the polling loop to send off new transactions
    pub async fn run(&self) -> Result<()> {
        let contract = self.contract.clone();
        loop {
            if let Some(tx) = self.next_transaction() {
                contract.forward(tx).await;
            }
            tokio::time::sleep(Duration::from_millis(TX_STATUS_POLL_MS)).await;
        }
    }
}

impl std::fmt::Display for TxPoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
