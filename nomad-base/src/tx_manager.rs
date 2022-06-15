use crate::NomadDB;
use color_eyre::Result;
use nomad_core::db::DbError;
use nomad_core::{PersistedTransaction, TxDispatchKind};

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
    pub fn submit_transaction(
        &self,
        tx: PersistedTransaction,
        dispatch_kind: TxDispatchKind,
    ) -> Result<(), DbError> {
        self.db.store_persisted_transaction(&tx)
    }
}

impl std::fmt::Display for TxManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
