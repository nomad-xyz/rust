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
pub trait TxForwarder: Send + Sync + std::fmt::Debug {
    /// Translate to chain-specific type
    async fn forward(&self, tx: PersistedTransaction)
        -> Result<TxOutcome, ChainCommunicationError>;
}

/// Interface for checking tx status via emitted events
#[async_trait]
pub trait TxEventStatus: Send + Sync + std::fmt::Debug {
    /// Get status of transaction via contract state
    async fn event_status(
        &self,
        tx: &PersistedTransaction,
    ) -> Result<TxOutcome, ChainCommunicationError>;
}

/// Interface for checking tx status via contract state
#[async_trait]
pub trait TxContractStatus: Send + Sync + std::fmt::Debug {
    /// Get status of transaction via contract state
    async fn contract_status(
        &self,
        tx: &PersistedTransaction,
    ) -> Result<TxOutcome, ChainCommunicationError>;
}
