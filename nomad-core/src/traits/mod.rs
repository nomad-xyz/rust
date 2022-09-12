mod encode;
mod home;
mod indexer;
mod replica;
mod signer;
mod xapp;

use async_trait::async_trait;
use color_eyre::Result;
use ethers::core::types::H256;
use std::{error::Error as StdError, fmt::Display};

use crate::{db::DbError, SignedUpdate};

pub use encode::*;
pub use home::*;
pub use indexer::*;
pub use replica::*;
pub use signer::*;
pub use xapp::*;

/// Box std error with send + sync
pub type BoxStdError = Box<dyn std::error::Error + Send + Sync>;

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

/// Interface for attributes shared by Home and Replica
#[async_trait]
pub trait Common: Sync + Send + std::fmt::Debug + std::fmt::Display {
    /// Chain-specific error type
    type Error: StdError + Send + Sync;

    /// Return an identifier (not necessarily unique) for the chain this
    /// contract is running on.
    fn name(&self) -> &str;

    /// Get the status of a transaction.
    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, Self::Error>;

    /// Fetch the current updater value
    async fn updater(&self) -> Result<H256, Self::Error>;

    /// Fetch the current state.
    async fn state(&self) -> Result<State, Self::Error>;

    /// Fetch the current root.
    async fn committed_root(&self) -> Result<H256, Self::Error>;

    /// Submit a signed update for inclusion
    async fn update(&self, update: &SignedUpdate) -> Result<TxOutcome, Self::Error>;

    /// Submit a double update for slashing
    async fn double_update(&self, double: &DoubleUpdate) -> Result<TxOutcome, Self::Error>;
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
