use crate::decodings::{NomadBase, NomadLightMerkleWrapper, NomadState};
use crate::SubstrateError;
use crate::{report_tx, utils, NomadOnlineClient, SubstrateSigner};
use async_trait::async_trait;
use color_eyre::{eyre::eyre, Result};
use ethers_core::types::{H160, H256, U256};
use futures::{stream::FuturesOrdered, StreamExt};
use std::sync::Arc;
use subxt::ext::scale_value::{self, Primitive, Value};
use subxt::{ext::sp_runtime::traits::Header, tx::ExtrinsicParams};
use subxt::{Config, OnlineClient};
use tracing::info;

use nomad_core::{
    accumulator::{Merkle, NomadLightMerkle},
    Common, CommonIndexer, DoubleUpdate, Home, HomeIndexer, Message, RawCommittedMessage,
    SignedUpdate, SignedUpdateWithMeta, State, TxOutcome, Update,
};

/// Substrate home indexer
#[derive(Clone)]
pub struct SubstrateHomeIndexer<T: Config>(NomadOnlineClient<T>);

impl<T> SubstrateHomeIndexer<T>
where
    T: Config,
{
    /// Instantiate a new SubstrateHomeIndexer object
    pub fn new(client: NomadOnlineClient<T>) -> Self {
        Self(client)
    }
}

impl<T> std::ops::Deref for SubstrateHomeIndexer<T>
where
    T: Config,
{
    type Target = OnlineClient<T>;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> std::fmt::Debug for SubstrateHomeIndexer<T>
where
    T: Config,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SubstrateHomeIndexer",)
    }
}

#[async_trait]
impl<T> CommonIndexer for SubstrateHomeIndexer<T>
where
    T: Config + Send + Sync,
    T::BlockNumber: std::convert::TryInto<u32> + Send + Sync,
{
    #[tracing::instrument(err, skip(self))]
    async fn get_block_number(&self) -> Result<u32> {
        let header = self.rpc().header(None).await?.unwrap();
        let u32_header = (*header.number()).try_into();

        if let Ok(h) = u32_header {
            Ok(h)
        } else {
            Err(eyre!("Failed to convert block number to u32"))
        }
    }

    #[tracing::instrument(err, skip(self))]
    async fn fetch_sorted_updates(&self, from: u32, to: u32) -> Result<Vec<SignedUpdateWithMeta>> {
        let mut futs = FuturesOrdered::new();
        for block_number in from..to {
            futs.push(self.0.fetch_sorted_updates_for_block(block_number))
        }

        // Flatten all Future<Output = Result<Vec<SignedUpdateWithMeta>>> into
        // single Vec<SignedUpdateWithMeta>
        Ok(futs
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect())
    }
}

#[async_trait]
impl<T> HomeIndexer for SubstrateHomeIndexer<T>
where
    T: Config + Send + Sync,
    T::BlockNumber: std::convert::TryInto<u32> + Send + Sync,
{
    #[tracing::instrument(err, skip(self))]
    async fn fetch_sorted_messages(&self, from: u32, to: u32) -> Result<Vec<RawCommittedMessage>> {
        let mut futs = FuturesOrdered::new();
        for block_number in from..to {
            futs.push(self.0.fetch_sorted_messages_for_block(block_number))
        }

        // Flatten all Future<Output = Result<Vec<RawCommittedMessage>>> into
        // single Vec<RawCommittedMessage>
        Ok(futs
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect())
    }
}

/// Substrate
#[derive(Clone)]
pub struct SubstrateHome<T: Config> {
    api: NomadOnlineClient<T>,
    signer: Arc<SubstrateSigner<T>>,
    domain: u32,
    name: String,
}

impl<T> SubstrateHome<T>
where
    T: Config,
{
    /// Instantiate a new SubstrateHome object
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

    /// Retrieve the home's base object from chain storage
    pub(crate) async fn base(&self) -> Result<NomadBase, SubstrateError> {
        let base_address = subxt::dynamic::storage_root("Home", "Base");
        let base_value = self.storage().fetch(&base_address, None).await?.unwrap();
        Ok(scale_value::serde::from_value(base_value)?)
    }

    /// Retrieve the home's base object from chain storage
    pub async fn tree(&self) -> Result<NomadLightMerkle, SubstrateError> {
        let tree_address = subxt::dynamic::storage_root("Home", "Tree");
        let tree_value = self.storage().fetch(&tree_address, None).await?.unwrap();
        let merkle_wrapper: NomadLightMerkleWrapper = scale_value::serde::from_value(tree_value)?;
        Ok(merkle_wrapper.into())
    }
}

impl<T> std::ops::Deref for SubstrateHome<T>
where
    T: Config,
{
    type Target = OnlineClient<T>;
    fn deref(&self) -> &Self::Target {
        self.api.deref()
    }
}

impl<T> std::fmt::Debug for SubstrateHome<T>
where
    T: Config,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubstrateHome {{ domain: {}, name: {} }}",
            self.domain, self.name,
        )
    }
}

impl<T> std::fmt::Display for SubstrateHome<T>
where
    T: Config,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubstrateHome {{ domain: {}, name: {} }}",
            self.domain, self.name,
        )
    }
}

#[async_trait]
impl<T> Common for SubstrateHome<T>
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
        unimplemented!("Have not implemented _status_ for substrate home")
    }

    #[tracing::instrument(err, skip(self))]
    async fn updater(&self) -> Result<H256, Self::Error> {
        let base = self.base().await?;
        let updater: H160 = base.updater.into();
        Ok(updater.into())
    }

    #[tracing::instrument(err, skip(self))]
    async fn state(&self) -> Result<State, Self::Error> {
        let base = self.base().await?;
        match base.state {
            NomadState::Active => Ok(nomad_core::State::Active),
            NomadState::Failed => Ok(nomad_core::State::Failed),
        }
    }

    #[tracing::instrument(err, skip(self))]
    async fn committed_root(&self) -> Result<H256, Self::Error> {
        let base = self.base().await?;
        Ok(base.committed_root.into())
    }

    #[tracing::instrument(err, skip(self, update), fields(update = %update))]
    async fn update(&self, update: &SignedUpdate) -> Result<TxOutcome, Self::Error> {
        let signed_update_value = utils::format_signed_update_value(update);
        let tx_payload = subxt::dynamic::tx("Home", "update", vec![signed_update_value]);

        info!(update = ?update, "Submitting update to chain.");
        report_tx!("update", self.api, self.signer, tx_payload)
    }

    #[tracing::instrument(err, skip(self))]
    async fn double_update(&self, _double: &DoubleUpdate) -> Result<TxOutcome, Self::Error> {
        unimplemented!("Double update deprecated for Substrate implementations")
    }
}

#[async_trait]
impl<T> Home for SubstrateHome<T>
where
    T: Config + Send + Sync,
    <<T as Config>::ExtrinsicParams as ExtrinsicParams<
        <T as Config>::Index,
        <T as Config>::Hash,
    >>::OtherParams: std::default::Default + Send + Sync,
    <T as Config>::Extrinsic: Send + Sync,
    <T as Config>::Hash: Into<H256>,
{
    fn local_domain(&self) -> u32 {
        self.domain
    }

    #[tracing::instrument(err, skip(self))]
    async fn nonces(&self, destination: u32) -> Result<u32, <Self as Common>::Error> {
        let nonce_address =
            subxt::dynamic::storage("Home", "Nonces", vec![Value::u128(destination as u128)]);
        let nonce_value = self
            .storage()
            .fetch(&nonce_address, None)
            .await?
            .unwrap_or_else(|| panic!("No nonce for destination {}", destination));
        Ok(scale_value::serde::from_value(nonce_value)?)
    }

    #[tracing::instrument(err, skip(self))]
    async fn dispatch(&self, message: &Message) -> Result<TxOutcome, <Self as Common>::Error> {
        let Message {
            destination,
            recipient,
            body,
        } = message;

        let destination_value = Value::u128(*destination as u128);
        let recipient_value = Value::primitive(Primitive::U256((*recipient).into()));
        let body_value = Value::from_bytes(body);

        let tx_payload = subxt::dynamic::tx(
            "Home",
            "dispatch",
            vec![destination_value, recipient_value, body_value],
        );

        info!(message = ?message, "Dispatching message to chain.");
        report_tx!("dispatch", self.api, self.signer, tx_payload)
    }

    async fn queue_length(&self) -> Result<U256, <Self as Common>::Error> {
        unimplemented!("Queue deprecated for Substrate implementations")
    }

    async fn queue_contains(&self, root: H256) -> Result<bool, <Self as Common>::Error> {
        let index_address =
            subxt::dynamic::storage("Home", "RootToIndex", vec![Value::from_bytes(&root)]);
        let index_value = self.storage().fetch(&index_address, None).await?;
        Ok(index_value.is_some())
    }

    #[tracing::instrument(err, skip(self), fields(hex_signature = %format!("0x{}", hex::encode(update.signature.to_vec()))))]
    async fn improper_update(
        &self,
        update: &SignedUpdate,
    ) -> Result<TxOutcome, <Self as Common>::Error> {
        let signed_update_value = utils::format_signed_update_value(update);
        let tx_payload = subxt::dynamic::tx("Home", "improper_update", vec![signed_update_value]);

        info!(update = ?update, "Dispatching improper update call to chain.");
        report_tx!("improper_update", self.api, self.signer, tx_payload)
    }

    #[tracing::instrument(err, skip(self))]
    async fn produce_update(&self) -> Result<Option<Update>, <Self as Common>::Error> {
        let committed_root: H256 = self.base().await?.committed_root.into();
        let new_root = self.tree().await?.root();

        Ok(if committed_root == new_root {
            None
        } else {
            Some(Update {
                home_domain: self.domain,
                previous_root: committed_root,
                new_root,
            })
        })
    }
}
