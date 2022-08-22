use crate::SubstrateError;
use crate::{NomadOnlineClient, SubstrateSigner};
use async_trait::async_trait;
use color_eyre::Result;
use ethers_core::types::H256;
use nomad_core::{
    Common, CommonIndexer, DoubleUpdate, SignedUpdate, SignedUpdateWithMeta, State, TxOutcome,
};
use std::sync::Arc;
use subxt::tx::ExtrinsicParams;
use subxt::{Config, OnlineClient};

/// Substrate home indexer
#[derive(Clone)]
pub struct SubstrateReplicaIndexer<T: Config>(NomadOnlineClient<T>);

impl<T> SubstrateReplicaIndexer<T>
where
    T: Config,
{
    /// Instantiate a new SubstrateReplicaIndexer object
    pub fn new(client: NomadOnlineClient<T>) -> Self {
        Self(client)
    }
}

impl<T> std::ops::Deref for SubstrateReplicaIndexer<T>
where
    T: Config,
{
    type Target = OnlineClient<T>;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> std::fmt::Debug for SubstrateReplicaIndexer<T>
where
    T: Config,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubstrateReplicaIndexer",)
    }
}

#[async_trait]
impl<T> CommonIndexer for SubstrateReplicaIndexer<T>
where
    T: Config + Send + Sync,
    T::BlockNumber: std::convert::TryInto<u32> + Send + Sync,
{
    #[tracing::instrument(err, skip(self))]
    async fn get_block_number(&self) -> Result<u32> {
        unimplemented!("Substrate replica not yet implemented")
    }

    #[tracing::instrument(err, skip(self))]
    async fn fetch_sorted_updates(
        &self,
        _from: u32,
        _to: u32,
    ) -> Result<Vec<SignedUpdateWithMeta>> {
        unimplemented!("Substrate replica not yet implemented")
    }
}

/// Substrate replica
#[derive(Clone)]
pub struct SubstrateReplica<T: Config> {
    api: NomadOnlineClient<T>,
    #[allow(dead_code)]
    signer: Arc<SubstrateSigner<T>>,
    domain: u32,
    name: String,
}

impl<T> SubstrateReplica<T>
where
    T: Config,
{
    /// Instantiate a new SubstrateReplica object
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

impl<T> std::ops::Deref for SubstrateReplica<T>
where
    T: Config,
{
    type Target = OnlineClient<T>;
    fn deref(&self) -> &Self::Target {
        self.api.deref()
    }
}

impl<T> std::fmt::Debug for SubstrateReplica<T>
where
    T: Config,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubstrateReplica {{ domain: {}, name: {} }}",
            self.domain, self.name,
        )
    }
}

impl<T> std::fmt::Display for SubstrateReplica<T>
where
    T: Config,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubstrateReplica {{ domain: {}, name: {} }}",
            self.domain, self.name,
        )
    }
}

#[async_trait]
impl<T> Common for SubstrateReplica<T>
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

    fn name(&self) -> &str {
        &self.name
    }

    #[tracing::instrument(err, skip(self))]
    async fn status(&self, _txid: H256) -> Result<Option<TxOutcome>, Self::Error> {
        unimplemented!("Substrate replica not yet implemented")
    }

    #[tracing::instrument(err, skip(self))]
    async fn updater(&self) -> Result<H256, Self::Error> {
        unimplemented!("Substrate replica not yet implemented")
    }

    #[tracing::instrument(err, skip(self))]
    async fn state(&self) -> Result<State, Self::Error> {
        unimplemented!("Substrate replica not yet implemented")
    }

    #[tracing::instrument(err, skip(self))]
    async fn committed_root(&self) -> Result<H256, Self::Error> {
        unimplemented!("Substrate replica not yet implemented")
    }

    #[tracing::instrument(err, skip(self, _update), fields(update = %_update))]
    async fn update(&self, _update: &SignedUpdate) -> Result<TxOutcome, Self::Error> {
        unimplemented!("Substrate replica not yet implemented")
    }

    #[tracing::instrument(err, skip(self))]
    async fn double_update(&self, _double: &DoubleUpdate) -> Result<TxOutcome, Self::Error> {
        unimplemented!("Double update deprecated for Substrate implementations")
    }
}
