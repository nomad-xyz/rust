use crate::avail_subxt_config::{avail::runtime_types::nomad_core::state::NomadState, *};
use anyhow::Result;
use async_trait::async_trait;
use avail::RuntimeApi;
use ethers_core::types::H256;
use std::sync::Arc;
use subxt::{AvailExtra, ClientBuilder, PairSigner, Signer};

use nomad_core::{
    ChainCommunicationError, Common, CommonIndexer, ContractLocator, DoubleUpdate, Home,
    HomeIndexer, Message, RawCommittedMessage, SignedUpdate, SignedUpdateWithMeta, State,
    TxOutcome, Update, UpdateMeta,
};

use crate::home::avail::runtime_types::nomad_base::NomadBase;

pub struct SubstrateHome {
    api: RuntimeApi<AvailConfig, AvailExtra<AvailConfig>>,
    signer: Arc<PairSigner<AvailConfig, AvailExtra<AvailConfig>, sp_core::ecdsa::Pair>>,
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
        signer: Arc<PairSigner<AvailConfig, AvailExtra<AvailConfig>, sp_core::ecdsa::Pair>>,
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
impl Common for SubstrateHome {
    fn name(&self) -> &str {
        &self.name
    }

    #[tracing::instrument(err, skip(self))]
    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
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
            .update((*update).into())
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

    #[tracing::instrument(err, skip(self, double), fields(double = %double))]
    async fn double_update(
        &self,
        double: &DoubleUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        unimplemented!("Double update deprecated for Substrate implementations")
    }
}
