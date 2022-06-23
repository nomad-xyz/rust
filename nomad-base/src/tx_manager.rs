use crate::NomadDB;
use color_eyre::Result;
use nomad_core::{
    ChainCommunicationError, NomadTxStatus, PersistedTransaction, TxDispatchKind, TxOutcome,
};
use std::time::Duration;

const TX_STATUS_POLL_MS: u64 = 100;

/// Transaction manager for handling PersistentTransaction
#[derive(Debug, Clone)]
pub struct TxManager {
    db: NomadDB,
}

impl TxManager {
    /// Create a new TxManager with a DB ref
    pub fn new(db: NomadDB) -> Self {
        Self { db }
    }

    /// Submit abstract transaction for sending and monitoring
    pub async fn submit_transaction(
        &self,
        tx: PersistedTransaction,
        dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        let counter = self
            .db
            .store_persisted_transaction(&tx.into())
            .map_err(|e| ChainCommunicationError::DbError(e))?;
        match dispatch_kind {
            TxDispatchKind::FireAndForget => Ok(TxOutcome::Dummy), // TODO(matthew):
            TxDispatchKind::WaitForResult => {
                let db = self.db.clone();
                tokio::spawn(async move {
                    loop {
                        let tx = db
                            .retrieve_persisted_transaction_by_counter(counter)?
                            .expect("tx missing from db");
                        if tx.confirm_event == NomadTxStatus::Dummy {
                            db.delete_persisted_transaction_by_counter(counter)?;
                            break Ok(TxOutcome::Dummy); // TODO(matthew):
                        }
                        tokio::time::sleep(Duration::from_millis(TX_STATUS_POLL_MS)).await;
                    }
                })
                .await
                .map_err(|e| ChainCommunicationError::NomadError(e.into()))?
            }
        }
    }
}

impl std::fmt::Display for TxManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
