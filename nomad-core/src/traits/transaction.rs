use crate::{ChainCommunicationError, PersistedTransaction};
use async_trait::async_trait;

/// Interface for chain-agnostic to chain-specifc transaction translators
#[async_trait]
pub trait TxTranslator {
    type Transaction;

    /// Translate to chain-specific type
    async fn convert(
        &self,
        tx: PersistedTransaction,
    ) -> Result<Self::Transaction, ChainCommunicationError>;
}
