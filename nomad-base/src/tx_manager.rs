use crate::NomadDB;
use color_eyre::Result;
use nomad_core::{
    ChainCommunicationError, PersistedTransaction, TxDispatchKind, TxOutcome,
};

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
        tx: impl Into<PersistedTransaction>,
        dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.db
            .store_persisted_transaction(&tx.into())
            .map_err(|e| ChainCommunicationError::DbError(e))
            .map(|_| TxOutcome::Dummy)
    }
}

impl std::fmt::Display for TxManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
