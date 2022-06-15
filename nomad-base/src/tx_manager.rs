use nomad_core::{PersistedTransaction, TxDispatchKind};
use crate::NomadDB;



/// Transaction manager for handling PersistentTransaction
#[derive(Debug, Clone)]
pub struct TxManager {
    db: NomadDB,
}

impl TxManager {
    /// Submit abstract transaction for sending and monitoring
    pub fn submit_transaction(&self, tx: PersistedTransaction, dispatch_kind: TxDispatchKind) {
        unimplemented!()
    }
}

impl std::fmt::Display for TxManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
