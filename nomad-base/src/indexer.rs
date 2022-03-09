use async_trait::async_trait;
use color_eyre::Result;
use nomad_core::{CommonIndexer, HomeIndexer, RawCommittedMessage, SignedUpdateWithMeta};
use nomad_test::mocks::MockIndexer;
use std::sync::Arc;

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
    async fn get_block_number(&self) -> Result<u32> {
        (*self).get_block_number().await
    }

    async fn fetch_sorted_updates(&self, from: u32, to: u32) -> Result<Vec<SignedUpdateWithMeta>> {
        (*self).fetch_sorted_updates(from, to).await
    }
}

/// Home/Replica CommonIndexerVariants type
#[derive(Debug)]
pub enum CommonIndexerVariants {
    /// Ethereum contract indexer
    Ethereum(Box<dyn CommonIndexer>),
    /// Mock indexer
    Mock(Box<dyn CommonIndexer>),
    /// Other indexer variant
    Other(Box<dyn CommonIndexer>),
}

#[async_trait]
impl CommonIndexer for CommonIndexerVariants {
    async fn get_block_number(&self) -> Result<u32> {
        match self {
            CommonIndexerVariants::Ethereum(indexer) => indexer.get_block_number().await,
            CommonIndexerVariants::Mock(indexer) => indexer.get_block_number().await,
            CommonIndexerVariants::Other(indexer) => indexer.get_block_number().await,
        }
    }

    async fn fetch_sorted_updates(&self, from: u32, to: u32) -> Result<Vec<SignedUpdateWithMeta>> {
        match self {
            CommonIndexerVariants::Ethereum(indexer) => {
                indexer.fetch_sorted_updates(from, to).await
            }
            CommonIndexerVariants::Mock(indexer) => indexer.fetch_sorted_updates(from, to).await,
            CommonIndexerVariants::Other(indexer) => indexer.fetch_sorted_updates(from, to).await,
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
    async fn get_block_number(&self) -> Result<u32> {
        (*self).get_block_number().await
    }

    async fn fetch_sorted_updates(&self, from: u32, to: u32) -> Result<Vec<SignedUpdateWithMeta>> {
        (*self).fetch_sorted_updates(from, to).await
    }
}

#[async_trait]
impl HomeIndexer for HomeIndexers {
    async fn fetch_sorted_messages(&self, from: u32, to: u32) -> Result<Vec<RawCommittedMessage>> {
        (*self).fetch_sorted_messages(from, to).await
    }
}

/// HomeIndexer type
#[derive(Debug)]
pub enum HomeIndexerVariants {
    /// Ethereum contract indexer
    Ethereum(Box<dyn HomeIndexer>),
    /// Mock indexer
    Mock(Box<dyn HomeIndexer>),
    /// Other indexer variant
    Other(Box<dyn HomeIndexer>),
}

#[async_trait]
impl CommonIndexer for HomeIndexerVariants {
    async fn get_block_number(&self) -> Result<u32> {
        match self {
            HomeIndexerVariants::Ethereum(indexer) => indexer.get_block_number().await,
            HomeIndexerVariants::Mock(indexer) => indexer.get_block_number().await,
            HomeIndexerVariants::Other(indexer) => indexer.get_block_number().await,
        }
    }

    async fn fetch_sorted_updates(&self, from: u32, to: u32) -> Result<Vec<SignedUpdateWithMeta>> {
        match self {
            HomeIndexerVariants::Ethereum(indexer) => indexer.fetch_sorted_updates(from, to).await,
            HomeIndexerVariants::Mock(indexer) => indexer.fetch_sorted_updates(from, to).await,
            HomeIndexerVariants::Other(indexer) => indexer.fetch_sorted_updates(from, to).await,
        }
    }
}

#[async_trait]
impl HomeIndexer for HomeIndexerVariants {
    async fn fetch_sorted_messages(&self, from: u32, to: u32) -> Result<Vec<RawCommittedMessage>> {
        match self {
            HomeIndexerVariants::Ethereum(indexer) => indexer.fetch_sorted_messages(from, to).await,
            HomeIndexerVariants::Mock(indexer) => indexer.fetch_sorted_messages(from, to).await,
            HomeIndexerVariants::Other(indexer) => indexer.fetch_sorted_messages(from, to).await,
        }
    }
}
