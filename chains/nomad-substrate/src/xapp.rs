use crate::SubstrateError;
use crate::{NomadOnlineClient, SubstrateSigner};
use async_trait::async_trait;
use color_eyre::Result;
use ethers_core::types::H256;
use nomad_core::{ConnectionManager, SignedFailureNotification, TxOutcome};
use nomad_types::NomadIdentifier;
use std::sync::Arc;
use subxt::tx::ExtrinsicParams;
use subxt::{Config, OnlineClient};

/// Substrate replica
#[derive(Clone)]
pub struct SubstrateConnectionManager<T: Config> {
    api: NomadOnlineClient<T>,
    #[allow(dead_code)]
    signer: Arc<SubstrateSigner<T>>,
    domain: u32,
    name: String,
}

impl<T> SubstrateConnectionManager<T>
where
    T: Config,
{
    /// Instantiate a new SubstrateConnectionManager object
    pub fn new(
        api: NomadOnlineClient<T>,
        signer: Arc<SubstrateSigner<T>>,
        domain: u32,
        name: &str,
    ) -> Self {
        Self {
            api,
            signer,
            domain,
            name: name.to_owned(),
        }
    }
}

impl<T> std::ops::Deref for SubstrateConnectionManager<T>
where
    T: Config,
{
    type Target = OnlineClient<T>;
    fn deref(&self) -> &Self::Target {
        self.api.deref()
    }
}

impl<T> std::fmt::Debug for SubstrateConnectionManager<T>
where
    T: Config,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubstrateConnectionManager {{ domain: {}, name: {} }}",
            self.domain, self.name,
        )
    }
}

impl<T> std::fmt::Display for SubstrateConnectionManager<T>
where
    T: Config,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubstrateConnectionManager {{ domain: {}, name: {} }}",
            self.domain, self.name,
        )
    }
}

#[async_trait]
impl<T> ConnectionManager for SubstrateConnectionManager<T>
where
    T: Config + Send + Sync,
    <<T as Config>::ExtrinsicParams as ExtrinsicParams<
        <T as Config>::Index,
        <T as Config>::Hash,
    >>::OtherParams: std::default::Default + Send + Sync,
    <T as Config>::Extrinsic: Send + Sync,
    <T as Config>::Hash: Into<H256>,
{
    type Error = SubstrateError;

    fn local_domain(&self) -> u32 {
        self.domain
    }

    #[tracing::instrument(err)]
    async fn is_replica(&self, _address: NomadIdentifier) -> Result<bool, Self::Error> {
        unimplemented!("Substrate connection manager not yet implemented")
    }

    #[tracing::instrument(err)]
    async fn watcher_permission(
        &self,
        _address: NomadIdentifier,
        _domain: u32,
    ) -> Result<bool, Self::Error> {
        unimplemented!("Substrate connection manager not yet implemented")
    }

    #[tracing::instrument(err)]
    async fn owner_enroll_replica(
        &self,
        _replica: NomadIdentifier,
        _domain: u32,
    ) -> Result<TxOutcome, Self::Error> {
        unimplemented!("Substrate connection manager not yet implemented")
    }

    #[tracing::instrument(err)]
    async fn owner_unenroll_replica(
        &self,
        _replica: NomadIdentifier,
    ) -> Result<TxOutcome, Self::Error> {
        unimplemented!("Substrate connection manager not yet implemented")
    }

    #[tracing::instrument(err)]
    async fn set_home(&self, _home: NomadIdentifier) -> Result<TxOutcome, Self::Error> {
        unimplemented!("Substrate connection manager not yet implemented")
    }

    #[tracing::instrument(err)]
    async fn set_watcher_permission(
        &self,
        _watcher: NomadIdentifier,
        _domain: u32,
        _access: bool,
    ) -> Result<TxOutcome, Self::Error> {
        unimplemented!("Substrate connection manager not yet implemented")
    }

    #[tracing::instrument(err)]
    async fn unenroll_replica(
        &self,
        _signed_failure: &SignedFailureNotification,
    ) -> Result<TxOutcome, Self::Error> {
        unimplemented!("Substrate connection manager not yet implemented")
    }
}
