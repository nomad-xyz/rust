use crate::{traits::TxOutcome, SignedFailureNotification};
use async_trait::async_trait;
use nomad_types::NomadIdentifier;
use std::error::Error as StdError;

/// Interface for on-chain XAppConnectionManager
#[async_trait]
pub trait ConnectionManager: Send + Sync + std::fmt::Debug {
    /// Chain-specific error type
    type Error: StdError + Send + Sync;

    /// Return the contract's local domain ID
    fn local_domain(&self) -> u32;

    /// Returns true if provided address is enrolled replica
    async fn is_replica(&self, address: NomadIdentifier) -> Result<bool, Self::Error>;

    /// Returns permission for address at given domain
    async fn watcher_permission(
        &self,
        address: NomadIdentifier,
        domain: u32,
    ) -> Result<bool, Self::Error>;

    /// onlyOwner function. Enrolls replica at given domain chain.
    async fn owner_enroll_replica(
        &self,
        replica: NomadIdentifier,
        domain: u32,
    ) -> Result<TxOutcome, Self::Error>;

    /// onlyOwner function. Unenrolls replica.
    async fn owner_unenroll_replica(
        &self,
        replica: NomadIdentifier,
    ) -> Result<TxOutcome, Self::Error>;

    /// onlyOwner function. Sets contract's home to provided home.
    async fn set_home(&self, home: NomadIdentifier) -> Result<TxOutcome, Self::Error>;

    /// onlyOwner function. Sets permission for watcher at given domain.
    async fn set_watcher_permission(
        &self,
        watcher: NomadIdentifier,
        domain: u32,
        access: bool,
    ) -> Result<TxOutcome, Self::Error>;

    /// Unenroll the replica at the given domain provided an updater address
    /// and `SignedFailureNotification` from a watcher
    async fn unenroll_replica(
        &self,
        signed_failure: &SignedFailureNotification,
    ) -> Result<TxOutcome, Self::Error>;
}
