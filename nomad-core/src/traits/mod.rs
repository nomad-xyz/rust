mod encode;
mod home;
mod indexer;
mod replica;
mod xapp;

use async_trait::async_trait;
use color_eyre::Result;
use ethers::{
    contract::ContractError,
    core::types::{TransactionReceipt, H256},
    providers::{Middleware, ProviderError},
};
use std::{error::Error as StdError, fmt::Display};

use crate::{db::DbError, NomadError, SignedUpdate};

pub use encode::*;
pub use home::*;
pub use indexer::*;
pub use replica::*;
pub use xapp::*;

/// Contract states
#[derive(Debug, PartialEq, Eq)]
pub enum State {
    /// Contract uninitialized
    Uninitialized,
    /// Contract is active
    Active,
    /// Contract has failed
    Failed,
}

/// Returned by `check_double_update` if double update exists
#[derive(Debug, Clone, PartialEq)]
pub struct DoubleUpdate(pub SignedUpdate, pub SignedUpdate);

impl Display for DoubleUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DoubleUpdate {{ ")?;
        write!(f, "left: {} ", &self.0)?;
        write!(f, "right: {} ", &self.1)?;
        write!(f, " }}")
    }
}

/// The result of a transaction
#[derive(Debug, Clone, Copy)]
pub struct TxOutcome {
    /// The txid
    pub txid: H256,
    // TODO: more? What can be abstracted across all chains?
}

impl TryFrom<TransactionReceipt> for TxOutcome {
    type Error = ChainCommunicationError;

    fn try_from(t: TransactionReceipt) -> Result<Self, Self::Error> {
        if t.status.unwrap().low_u32() == 1 {
            Ok(Self {
                txid: t.transaction_hash,
            })
        } else {
            Err(ChainCommunicationError::NotExecuted(t.transaction_hash))
        }
    }
}

/// ChainCommunicationError contains errors returned when attempting to
/// call a chain or dispatch a transaction
#[derive(Debug, thiserror::Error)]
pub enum ChainCommunicationError {
    /// Nomad Error
    #[error("{0}")]
    NomadError(#[from] NomadError),
    /// Contract Error
    #[error("{0}")]
    ContractError(Box<dyn StdError + Send + Sync>),
    /// Provider Error
    #[error("{0}")]
    ProviderError(#[from] ProviderError),
    /// A transaction was dropped from the mempool
    #[error("Transaction dropped from mempool {0:?}")]
    DroppedError(H256),
    /// A transaction was not executed successfully
    #[error("Transaction was not executed successfully {0:?}")]
    NotExecuted(H256),
    /// General transaction submission error
    #[error("Transaction was not submitted to chain successfully {0:?}")]
    TxSubmissionError(Box<dyn StdError + Send + Sync>),
    /// Any other error
    #[error("{0}")]
    CustomError(#[from] Box<dyn StdError + Send + Sync>),
}

impl<M> From<ContractError<M>> for ChainCommunicationError
where
    M: Middleware + 'static,
{
    fn from(e: ContractError<M>) -> Self {
        Self::ContractError(Box::new(e))
    }
}

/// Interface for attributes shared by Home and Replica
#[async_trait]
pub trait Common: Sync + Send + std::fmt::Debug {
    /// Return an identifier (not necessarily unique) for the chain this
    /// contract is running on.
    fn name(&self) -> &str;

    /// Get the status of a transaction.
    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError>;

    /// Fetch the current updater value
    async fn updater(&self) -> Result<H256, ChainCommunicationError>;

    /// Fetch the current state.
    async fn state(&self) -> Result<State, ChainCommunicationError>;

    /// Fetch the current root.
    async fn committed_root(&self) -> Result<H256, ChainCommunicationError>;

    /// Submit a signed update for inclusion
    async fn update(&self, update: &SignedUpdate) -> Result<TxOutcome, ChainCommunicationError>;

    /// Submit a double update for slashing
    async fn double_update(
        &self,
        double: &DoubleUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError>;
}

/// Interface for retrieving event data emitted by both the home and replica
#[async_trait]
pub trait CommonEvents: Common + Send + Sync + std::fmt::Debug {
    /// Fetch the first signed update building off of `old_root`. If `old_root`
    /// was never accepted or has never been updated, this will return `Ok(None )`.
    /// This should fetch events from the chain API
    async fn signed_update_by_old_root(
        &self,
        old_root: H256,
    ) -> Result<Option<SignedUpdate>, DbError>;

    /// Fetch the first signed update with a new root of `new_root`. If update
    /// has not been produced, this will return `Ok(None)`. This should fetch
    /// events from the chain API
    async fn signed_update_by_new_root(
        &self,
        new_root: H256,
    ) -> Result<Option<SignedUpdate>, DbError>;
}

#[cfg(test)]
mod test {
    use ethers::prelude::U64;

    use super::*;

    #[tokio::test]
    async fn turning_transaction_receipt_into_tx_outcome() {
        let mut receipt = TransactionReceipt::default();
        receipt.status = Some(U64::from(0));
        let tx_outcome: Result<TxOutcome, ChainCommunicationError> = receipt.try_into();
        assert!(
            tx_outcome.is_err(),
            "Turning failed transaction receipt into errored tx outcome not succeeded"
        );

        let receipt = TransactionReceipt {
            status: Some(U64::from(1)),
            ..Default::default()
        };
        let tx_outcome: Result<TxOutcome, ChainCommunicationError> = receipt.try_into();
        assert!(
            tx_outcome.is_ok(),
            "Turning successeeded transaction receipt into successful tx outcome not succeeded"
        );
    }
}
