use crate::PersistedTransaction;

/// Interface for chain-agnostic to chain-specifc transaction translators
pub trait TxTranslator<T> {
    /// Translate to chain-specific type
    fn convert(&self, tx: &PersistedTransaction) -> T;
}
