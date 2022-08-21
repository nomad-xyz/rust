use crate::SubstrateError;
use crate::{
    avail_subxt_config::avail::home, report_tx, utils, NomadBase, NomadState, SubstrateSigner,
};
use async_trait::async_trait;
use color_eyre::{eyre::eyre, Result};
use ethers_core::types::Signature;
use ethers_core::types::{H256, U256};
use futures::{stream::FuturesOrdered, StreamExt};
use std::sync::Arc;
use subxt::ext::scale_value::{self, Value};
use subxt::Config;
use subxt::{ext::sp_runtime::traits::Header, tx::ExtrinsicParams, OnlineClient};
use tracing::info;

use nomad_core::{
    accumulator::{Merkle, NomadLightMerkle},
    Common, CommonIndexer, DoubleUpdate, Home, HomeIndexer, Message, RawCommittedMessage,
    SignedUpdate, SignedUpdateWithMeta, State, TxOutcome, Update, UpdateMeta,
};

/// Substrate home indexer
#[derive(Clone)]
pub struct SubstrateHomeIndexer<T: Config>(OnlineClient<T>);

impl<T> SubstrateHomeIndexer<T>
where
    T: Config,
{
    /// Instantiate a new SubstrateHomeIndexer object
    pub fn new(client: OnlineClient<T>) -> Self {
        Self(client)
    }
}

impl<T> std::ops::Deref for SubstrateHomeIndexer<T>
where
    T: Config,
{
    type Target = OnlineClient<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
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
        // Create future for fetching block hashes for range
        let numbers_and_hash_futs: FuturesOrdered<_> = (from..to)
            .map(|n| async move { (n, self.rpc().block_hash(Some(n.into())).await) })
            .collect();

        // Await and block hash requests
        let numbers_and_hashes: Vec<_> = numbers_and_hash_futs
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|r| (r.0, r.1.unwrap().unwrap())) // TODO: is this safe to unwrap  (log RPC err)?
            .collect();

        // Get futures for events for each block's hash
        let numbers_and_event_futs: FuturesOrdered<_> = numbers_and_hashes
            .into_iter()
            .map(|(n, h)| async move { (n, self.events().at(Some(h)).await) })
            .collect();

        // Await event requests and filter only update events
        let numbers_and_update_events: Vec<_> = numbers_and_event_futs
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|(n, r)| {
                let events_for_block = r.unwrap();
                events_for_block
                    .find::<home::events::Update>() // TODO: remove dep on avail metadata and break into custom struct that impls Decode and StaticEvent
                    .map(|r| (n, r.unwrap())) // TODO: is this safe to unwrap (log RPC err)?
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        // TODO: sort events

        // Map update events into SignedUpdates with meta
        Ok(numbers_and_update_events
            .into_iter()
            .map(|(n, e)| {
                let signature = Signature::try_from(e.signature.as_ref())
                    .expect("chain accepted invalid signature");

                SignedUpdateWithMeta {
                    signed_update: SignedUpdate {
                        update: nomad_core::Update {
                            home_domain: e.home_domain,
                            previous_root: e.previous_root,
                            new_root: e.new_root,
                        },
                        signature,
                    },
                    metadata: UpdateMeta {
                        block_number: n as u64,
                        timestamp: None,
                    },
                }
            })
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
        // Create future for fetching block hashes for range
        let hash_futs: FuturesOrdered<_> = (from..to)
            .map(|n| self.rpc().block_hash(Some(n.into())))
            .collect();

        // Await and block hash requests
        let hashes: Vec<_> = hash_futs
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|r| r.unwrap().unwrap()) // TODO: is this safe to unwrap  (log RPC err)?
            .collect();

        // Get futures for events for each block's hash
        let event_futs: FuturesOrdered<_> = hashes
            .into_iter()
            .map(|h| self.events().at(Some(h)))
            .collect();

        // Await event requests and filter only dispatch events
        let dispatch_events: Vec<_> = event_futs
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|r| {
                let events_for_block = r.unwrap();
                events_for_block
                    .find::<home::events::Dispatch>() // TODO: remove dep on avail metadata and break into custom struct that impls Decode and StaticEvent
                    .map(|r| r.unwrap()) // TODO: is this safe to unwrap (log RPC err)?
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        // TODO: sort events

        // Map update events into SignedUpdates with meta
        Ok(dispatch_events
            .into_iter()
            .map(|e| RawCommittedMessage {
                leaf_index: e.leaf_index,
                committed_root: e.committed_root,
                message: e.message,
            })
            .collect())
    }
}

/// Substrate
#[derive(Clone)]
pub struct SubstrateHome<T: Config> {
    api: OnlineClient<T>,
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
        api: OnlineClient<T>,
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
    pub async fn base(&self) -> Result<NomadBase> {
        let base_address = subxt::dynamic::storage_root("Home", "Base");
        let base_value = self.storage().fetch(&base_address, None).await?.unwrap();
        Ok(scale_value::serde::from_value(base_value)?)
    }
}

impl<T> std::ops::Deref for SubstrateHome<T>
where
    T: Config,
{
    type Target = OnlineClient<T>;
    fn deref(&self) -> &Self::Target {
        &self.api
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
        let base = self.base().await.unwrap();
        let updater = base.updater;
        Ok(updater.into()) // H256 is primitive-types 0.11.1 not 0.10.1
    }

    #[tracing::instrument(err, skip(self))]
    async fn state(&self) -> Result<State, Self::Error> {
        let base = self.base().await.unwrap();
        match base.state {
            NomadState::Active => Ok(nomad_core::State::Active),
            NomadState::Failed => Ok(nomad_core::State::Failed),
        }
    }

    #[tracing::instrument(err, skip(self))]
    async fn committed_root(&self) -> Result<H256, Self::Error> {
        let base = self.base().await.unwrap();
        Ok(base.committed_root)
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
        let nonce_value = self.storage().fetch(&nonce_address, None).await?.unwrap();
        Ok(scale_value::serde::from_value(nonce_value)
            .expect("failed to decode nonce from home::nonces call"))
    }

    #[tracing::instrument(err, skip(self))]
    async fn dispatch(&self, message: &Message) -> Result<TxOutcome, <Self as Common>::Error> {
        let Message {
            destination,
            recipient,
            body,
        } = message;

        let destination_value = Value::u128(*destination as u128);
        let recipient_value = Value::from_bytes(recipient);
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
        let committed_root = self.base().await.unwrap().committed_root;

        let tree_address = subxt::dynamic::storage_root("Home", "Tree");
        let tree_value = self.storage().fetch(&tree_address, None).await?.unwrap();
        let tree: NomadLightMerkle = scale_value::serde::from_value(tree_value).unwrap();

        let num_elements = tree.count();
        let root_address = subxt::dynamic::storage(
            "Home",
            "IndexToRoot",
            vec![Value::u128(num_elements as u128)],
        );
        let root_value = self.storage().fetch(&root_address, None).await?;

        Ok(root_value.map(|r| {
            let root: H256 = scale_value::serde::from_value(r).unwrap();
            Update {
                home_domain: self.domain,
                previous_root: committed_root,
                new_root: root,
            }
        }))
    }
}
