use async_trait::async_trait;
use color_eyre::Result;
use nomad_core::{CommonIndexer, HomeIndexer, RawCommittedMessage, SignedUpdateWithMeta};
use nomad_test::mocks::MockIndexer;
use std::{ops::Deref, sync::Arc};

use crate::ChainCommunicationError;

#[derive(Debug, Clone)]
/// Arc wrapper for HomeVariants enum
pub struct CommonIndexers(Arc<CommonIndexerVariants>);

impl From<CommonIndexerVariants> for CommonIndexers {
    fn from(common_indexers: CommonIndexerVariants) -> Self {
        Self(Arc::new(common_indexers))
    }
}

impl std::ops::Deref for CommonIndexers {
    type Target = Arc<CommonIndexerVariants>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CommonIndexers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<MockIndexer> for CommonIndexers {
    fn from(mock_indexer: MockIndexer) -> Self {
        Self(Arc::new(CommonIndexerVariants::Mock(Box::new(
            mock_indexer,
        ))))
    }
}

#[async_trait]
impl CommonIndexer for CommonIndexers {
    type Error = ChainCommunicationError;

    async fn get_block_number(&self) -> Result<u32, Self::Error> {
        self.deref().get_block_number().await
    }

    async fn fetch_sorted_updates(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<SignedUpdateWithMeta>, Self::Error> {
        self.deref().fetch_sorted_updates(from, to).await
    }
}

/// Home/Replica CommonIndexerVariants type
#[derive(Debug)]
pub enum CommonIndexerVariants {
    /// Ethereum contract indexer
    Ethereum(Box<dyn CommonIndexer<Error = nomad_ethereum::EthereumError>>),
    /// Mock indexer
    Mock(Box<MockIndexer>),
}

#[async_trait]
impl CommonIndexer for CommonIndexerVariants {
    type Error = ChainCommunicationError;

    async fn get_block_number(&self) -> Result<u32, Self::Error> {
        match self {
            CommonIndexerVariants::Ethereum(indexer) => Ok(indexer.get_block_number().await?),
            CommonIndexerVariants::Mock(indexer) => Ok(indexer.get_block_number().await?),
        }
    }

    async fn fetch_sorted_updates(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<SignedUpdateWithMeta>, Self::Error> {
        match self {
            CommonIndexerVariants::Ethereum(indexer) => {
                Ok(indexer.fetch_sorted_updates(from, to).await?)
            }
            CommonIndexerVariants::Mock(indexer) => {
                Ok(indexer.fetch_sorted_updates(from, to).await?)
            }
        }
    }
}

#[derive(Debug, Clone)]
/// Arc wrapper for home indexer variants
pub struct HomeIndexers(Arc<HomeIndexerVariants>);

impl From<HomeIndexerVariants> for HomeIndexers {
    fn from(homes_indexers: HomeIndexerVariants) -> Self {
        Self(Arc::new(homes_indexers))
    }
}

impl std::ops::Deref for HomeIndexers {
    type Target = Arc<HomeIndexerVariants>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for HomeIndexers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<MockIndexer> for HomeIndexers {
    fn from(mock_indexer: MockIndexer) -> Self {
        Self(Arc::new(HomeIndexerVariants::Mock(Box::new(mock_indexer))))
    }
}

#[async_trait]
impl CommonIndexer for HomeIndexers {
    type Error = ChainCommunicationError;

    async fn get_block_number(&self) -> Result<u32, Self::Error> {
        self.deref().get_block_number().await
    }

    async fn fetch_sorted_updates(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<SignedUpdateWithMeta>, Self::Error> {
        self.deref().fetch_sorted_updates(from, to).await
    }
}

#[async_trait]
impl HomeIndexer for HomeIndexers {
    async fn fetch_sorted_messages(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<RawCommittedMessage>, <Self as CommonIndexer>::Error> {
        self.deref().fetch_sorted_messages(from, to).await
    }
}

/// HomeIndexer type
#[derive(Debug)]
pub enum HomeIndexerVariants {
    /// Ethereum contract indexer
    Ethereum(Box<dyn HomeIndexer<Error = nomad_ethereum::EthereumError>>),
    /// Substrate contract indexer
    Substrate(Box<dyn HomeIndexer<Error = nomad_substrate::SubstrateError>>),
    /// Mock indexer
    Mock(Box<MockIndexer>),
}

#[async_trait]
impl CommonIndexer for HomeIndexerVariants {
    type Error = ChainCommunicationError;

    async fn get_block_number(&self) -> Result<u32, Self::Error> {
        match self {
            HomeIndexerVariants::Ethereum(indexer) => Ok(indexer.get_block_number().await?),
            HomeIndexerVariants::Substrate(indexer) => Ok(indexer.get_block_number().await?),
            HomeIndexerVariants::Mock(indexer) => Ok(indexer.get_block_number().await?),
        }
    }

    async fn fetch_sorted_updates(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<SignedUpdateWithMeta>, Self::Error> {
        match self {
            HomeIndexerVariants::Ethereum(indexer) => {
                Ok(indexer.fetch_sorted_updates(from, to).await?)
            }
            HomeIndexerVariants::Substrate(indexer) => {
                Ok(indexer.fetch_sorted_updates(from, to).await?)
            }
            HomeIndexerVariants::Mock(indexer) => {
                Ok(indexer.fetch_sorted_updates(from, to).await?)
            }
        }
    }
}

#[async_trait]
impl HomeIndexer for HomeIndexerVariants {
    async fn fetch_sorted_messages(
        &self,
        from: u32,
        to: u32,
    ) -> Result<Vec<RawCommittedMessage>, <Self as CommonIndexer>::Error> {
        match self {
            HomeIndexerVariants::Ethereum(indexer) => {
                Ok(indexer.fetch_sorted_messages(from, to).await?)
            }
            HomeIndexerVariants::Substrate(indexer) => {
                Ok(indexer.fetch_sorted_messages(from, to).await?)
            }
            HomeIndexerVariants::Mock(indexer) => {
                Ok(indexer.fetch_sorted_messages(from, to).await?)
            }
        }
    }
}
