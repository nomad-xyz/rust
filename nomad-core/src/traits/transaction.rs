use crate::{ChainCommunicationError, PersistedTransaction, TxOutcome};
use async_trait::async_trait;

/// Interface for chain-agnostic to chain-specifc transaction translators
#[async_trait]
pub trait TxTranslator {
    /// Concrete transaction type
    type Transaction;

    /// Translate to chain-specific type
    async fn convert(
        &self,
        tx: PersistedTransaction,
    ) -> Result<Self::Transaction, ChainCommunicationError>;
}

/// Interface for submitting PersistentTransaction to a contract
#[async_trait]
pub trait TxSender: Send + Sync + std::fmt::Debug {
    /// Translate to chain-specific type
    async fn send(&self, tx: PersistedTransaction) -> Result<TxOutcome, ChainCommunicationError>;
}

/// Interface for checking tx status via emitted events
#[async_trait]
pub trait TxEventStatus {
    /// Get status of transaction via contract state
    async fn status(&self, tx: &PersistedTransaction)
        -> Result<TxOutcome, ChainCommunicationError>;
}

/// Interface for checking tx status via contract state
#[async_trait]
pub trait TxContractStatus {
    /// Get status of transaction via contract state
    async fn status(&self, tx: &PersistedTransaction)
        -> Result<TxOutcome, ChainCommunicationError>;
}
