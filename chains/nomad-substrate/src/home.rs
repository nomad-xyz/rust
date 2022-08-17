use crate::avail_subxt_config::{
    avail::home, avail::runtime_types::nomad_core::state::NomadState, *,
};
use async_trait::async_trait;
use avail::RuntimeApi;
use color_eyre::Result;
use ethers_core::types::Signature;
use ethers_core::types::H256;
use futures::{stream::FuturesOrdered, StreamExt};
use subxt::{AvailExtra, ClientBuilder, PairSigner, Signer};

use nomad_core::{
    ChainCommunicationError, Common, CommonIndexer, ContractLocator, DoubleUpdate, Home,
    HomeIndexer, Message, RawCommittedMessage, SignedUpdate, SignedUpdateWithMeta, State,
    TxOutcome, Update, UpdateMeta,
};

use crate::home::avail::runtime_types::nomad_base::NomadBase;

pub struct SubstrateHome {
    api: RuntimeApi<AvailConfig, AvailExtra<AvailConfig>>,
    signer: PairSigner<AvailConfig, AvailExtra<AvailConfig>, sp_core::ecdsa::Pair>,
    domain: u32,
    name: String,
}

impl std::ops::Deref for SubstrateHome {
    type Target = RuntimeApi<AvailConfig, AvailExtra<AvailConfig>>;
    fn deref(&self) -> &Self::Target {
        &self.api
    }
}

impl SubstrateHome {
    pub async fn new(
        api: RuntimeApi<AvailConfig, AvailExtra<AvailConfig>>,
        signer: PairSigner<AvailConfig, AvailExtra<AvailConfig>, sp_core::ecdsa::Pair>,
        domain: u32,
        name: &str,
    ) -> Result<Self> {
        Ok(Self {
            api,
            signer,
            domain,
            name: name.to_owned(),
        })
    }

    pub async fn base(&self) -> Result<NomadBase> {
        Ok(self.api.storage().home().base(None).await?)
    }

    // pub async fn tree(&self) -> Result<LightMerkle> {
    //     Ok(self.0.storage().home().tree(None).await?)
    // }

    // pub async fn nonces(&self, domain: u32) -> Result<u32> {
    //     Ok(self
    //         .0
    //         .storage()
    //         .home()
    //         .nonces(&domain, None)
    //         .await?
    //         .unwrap_or_default())
    // }

    // pub async fn index_to_root(&self, index: u32) -> Result<Option<H256>> {
    //     Ok(self.0.storage().home().index_to_root(&index, None).await?)
    // }

    // pub async fn root_to_index(&self, root: H256) -> Result<Option<u32>> {
    //     Ok(self.0.storage().home().root_to_index(&root, None).await?)
    // }
}

impl std::fmt::Debug for SubstrateHome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubstrateHome {{ domain: {}, name: {} }}",
            self.domain, self.name,
        )
    }
}

impl std::fmt::Display for SubstrateHome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubstrateHome {{ domain: {}, name: {} }}",
            self.domain, self.name,
        )
    }
}

#[async_trait]
impl CommonIndexer for SubstrateHome {
    #[tracing::instrument(err, skip(self))]
    async fn get_block_number(&self) -> Result<u32> {
        let header = self.api.client.rpc().header(None).await?.unwrap();
        Ok(header.number)
    }

    #[tracing::instrument(err, skip(self))]
    async fn fetch_sorted_updates(&self, from: u32, to: u32) -> Result<Vec<SignedUpdateWithMeta>> {
        // Create future for fetching block hashes for range
        let numbers_and_hash_futs: FuturesOrdered<_> = (from..to)
            .map(|n| async move { (n, self.api.client.rpc().block_hash(Some(n.into())).await) })
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
            .map(|(n, h)| async move {
                let events_api = self.api.events();
                (n, events_api.at(h).await)
            })
            .collect();

        // Await event requests and filter only update events
        let numbers_and_update_events: Vec<_> = numbers_and_event_futs
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|(n, r)| {
                let events_for_block = r.unwrap();
                events_for_block
                    .find::<home::events::Update>()
                    .map(|r| (n, r.unwrap())) // TODO: is this safe to unwrap (log RPC err)?
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        // Map update events into SignedUpdates with meta
        Ok(numbers_and_update_events
            .into_iter()
            .map(|(n, e)| {
                let signature = Signature::try_from(e.signature.as_ref())
                    .expect("chain accepted invalid signature");

                nomad_core::SignedUpdateWithMeta {
                    signed_update: nomad_core::SignedUpdate {
                        update: nomad_core::Update {
                            home_domain: e.home_domain,
                            previous_root: e.previous_root,
                            new_root: e.new_root,
                        },
                        signature,
                    },
                    metadata: nomad_core::UpdateMeta {
                        block_number: n as u64,
                        timestamp: None,
                    },
                }
            })
            .collect())
    }
}

#[async_trait]
impl Common for SubstrateHome {
    fn name(&self) -> &str {
        &self.name
    }

    #[tracing::instrument(err, skip(self))]
    async fn status(&self, _txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        unimplemented!("Have not implemented _status_ for substrate home")
    }

    #[tracing::instrument(err, skip(self))]
    async fn updater(&self) -> Result<H256, ChainCommunicationError> {
        let base = self.base().await.unwrap();
        let updater = base.updater;
        Ok(updater.into()) // H256 is primitive-types 0.11.1 not 0.10.1
    }

    #[tracing::instrument(err, skip(self))]
    async fn state(&self) -> Result<State, ChainCommunicationError> {
        let base = self.base().await.unwrap();
        match base.state {
            NomadState::Active => Ok(nomad_core::State::Active),
            NomadState::Failed => Ok(nomad_core::State::Failed),
        }
    }

    #[tracing::instrument(err, skip(self))]
    async fn committed_root(&self) -> Result<H256, ChainCommunicationError> {
        let base = self.base().await.unwrap();
        Ok(base.committed_root)
    }

    #[tracing::instrument(err, skip(self, update), fields(update = %update))]
    async fn update(&self, update: &SignedUpdate) -> Result<TxOutcome, ChainCommunicationError> {
        let res = self
            .tx()
            .home()
            .update(update.clone().into())
            .sign_and_submit_then_watch(&self.signer)
            .await
            .unwrap()
            .wait_for_finalized_success()
            .await
            .unwrap();

        Ok(TxOutcome {
            txid: res.extrinsic_hash(),
        })
    }

    #[tracing::instrument(err, skip(self))]
    async fn double_update(
        &self,
        _double: &DoubleUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        unimplemented!("Double update deprecated for Substrate implementations")
    }
}
