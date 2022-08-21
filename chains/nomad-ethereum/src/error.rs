use crate::gelato::GelatoError;
use ethers::core::types::H256;
use ethers::prelude::{ContractError, Middleware, ProviderError};
use std::error::Error as StdError;

/// Ethereum-specific error wrapper
#[derive(Debug, thiserror::Error)]
pub enum EthereumError {
    /// Ethers provider error
    #[error("{0}")]
    ProviderError(#[from] ProviderError),
    /// Ethers contract error
    #[error("{0}")]
    ContractError(Box<dyn StdError + Send + Sync>),
    /// Middleware error
    #[error("{0}")]
    MiddlewareError(Box<dyn StdError + Send + Sync>),
    /// Gelato client error
    #[error("{0}")]
    GelatoError(#[from] GelatoError),
    /// A transaction was dropped from the mempool
    #[error("Transaction dropped from mempool {0:?}")]
    DroppedError(H256),
    /// Transaction was not executed successfully
    #[error("Transaction was not executed successfully {0:?}")]
    TxNotExecuted(H256),
    /// Any other error
    #[error("{0}")]
    CustomError(#[from] Box<dyn StdError + Send + Sync>),
}

impl<M> From<ContractError<M>> for EthereumError
where
    M: Middleware + 'static,
{
    fn from(e: ContractError<M>) -> Self {
        Self::ContractError(e.into())
    }
}
