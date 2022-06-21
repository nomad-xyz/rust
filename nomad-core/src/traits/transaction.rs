use crate::PersistedTransaction;

/// Interface for chain-agnostic to chain-specifc transaction translators
pub trait TxTranslator {
    type Transaction;

    /// Translate to chain-specific type
    fn convert(&self, tx: PersistedTransaction) -> Self::Transaction;
}
