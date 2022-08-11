use crate::avail_subxt_config::{
    avail::runtime_types::{merkle::light::LightMerkle, nomad_base::NomadBase},
    *,
};
use anyhow::Result;
use avail::RuntimeApi;
use subxt::{sp_core::H256, AvailExtra, ClientBuilder};

pub struct SubstrateHome(RuntimeApi<AvailConfig, AvailExtra<AvailConfig>>);

impl std::ops::Deref for SubstrateHome {
    type Target = RuntimeApi<AvailConfig, AvailExtra<AvailConfig>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SubstrateHome {
    pub async fn new(url: &str) -> Result<Self> {
        Ok(Self(
            ClientBuilder::new()
                .set_url(url)
                .build()
                .await?
                .to_runtime_api::<avail::RuntimeApi<AvailConfig, AvailExtra<AvailConfig>>>(),
        ))
    }

    pub async fn base(&self) -> Result<NomadBase> {
        Ok(self.0.storage().home().base(None).await?)
    }

    pub async fn tree(&self) -> Result<LightMerkle> {
        Ok(self.0.storage().home().tree(None).await?)
    }

    pub async fn nonces(&self, domain: u32) -> Result<u32> {
        Ok(self
            .0
            .storage()
            .home()
            .nonces(&domain, None)
            .await?
            .unwrap_or_default())
    }

    pub async fn index_to_root(&self, index: u32) -> Result<Option<H256>> {
        Ok(self.0.storage().home().index_to_root(&index, None).await?)
    }

    pub async fn root_to_index(&self, root: H256) -> Result<Option<u32>> {
        Ok(self.0.storage().home().root_to_index(&root, None).await?)
    }
}
