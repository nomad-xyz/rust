use crate::avail_subxt_config::*;
use anyhow::Result;
use async_trait::async_trait;
use avail::RuntimeApi;
use subxt::{sp_core::H256, AvailExtra, ClientBuilder};

use nomad_core::{
    ChainCommunicationError, Common, CommonIndexer, ContractLocator, DoubleUpdate, Home,
    HomeIndexer, Message, RawCommittedMessage, SignedUpdate, SignedUpdateWithMeta, State,
    TxOutcome, Update, UpdateMeta,
};

use crate::home::avail::runtime_types::nomad_base::NomadBase;

pub struct SubstrateHome {
    api: RuntimeApi<AvailConfig, AvailExtra<AvailConfig>>,
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
    pub async fn new(url: &str, domain: u32, name: &str) -> Result<Self> {
        let api = ClientBuilder::new()
            .set_url(url)
            .build()
            .await?
            .to_runtime_api::<avail::RuntimeApi<AvailConfig, AvailExtra<AvailConfig>>>();

        Ok(Self {
            api,
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

// #[async_trait]
// impl Common for SubstrateHome {
//     fn name(&self) ->  &str {
//         &self.name
//     }

//     #[tracing::instrument(err, skip(self))]
//     async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
//         unimplemented!("Have not implemented _status_ for substrate home")
//     }

//     #[tracing::instrument(err, skip(self))]
//     async fn updater(&self) -> Result<H256, ChainCommunicationError> {
//         let base = self.base().await?;
//         Ok(base.updater.into())
//     }
// }
