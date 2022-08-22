use ethers_core::types::H256;
use std::error::Error as StdError;
use subxt::{ext::scale_value, Error as SubxtError};

/// Substrate-specific error wrapper
#[derive(Debug, thiserror::Error)]
pub enum SubstrateError {
    /// A transaction was not executed successfully
    #[error("Transaction was not executed successfully {0:?}")]
    TxNotExecuted(H256),
    /// Substrate provider error
    #[error("{0}")]
    ProviderError(#[from] SubxtError),
    /// Scale value deserialization error
    #[error("{0}")]
    DeserializationError(#[from] scale_value::serde::DeserializerError),
    /// Any other error
    #[error("{0}")]
    CustomError(#[from] Box<dyn StdError + Send + Sync>),
}
