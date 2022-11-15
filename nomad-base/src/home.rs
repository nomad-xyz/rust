use crate::{ChainCommunicationError, ContractSync, HomeIndexers, NomadDB};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use ethers::core::types::{H256, U256};
use nomad_core::{
    db::DbError, Common, CommonEvents, DoubleUpdate, Home, HomeEvents, Message,
    RawCommittedMessage, SignedUpdate, State, TxOutcome, Update,
};
use nomad_ethereum::EthereumHome;
use nomad_test::mocks::MockHomeContract;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::instrument;

/// Caching replica type
#[derive(Debug)]
pub struct CachingHome {
    home: Homes,
    contract_sync: ContractSync<HomeIndexers>,
    db: NomadDB,
}

impl std::fmt::Display for CachingHome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CachingHome {}", self.home)
    }
}

impl CachingHome {
    /// Instantiate new CachingHome
    pub fn new(home: Homes, contract_sync: ContractSync<HomeIndexers>, db: NomadDB) -> Self {
        Self {
            home,
            contract_sync,
            db,
        }
    }

    /// Return handle on home object
    pub fn home(&self) -> Homes {
        self.home.clone()
    }

    /// Return handle on NomadDB
    pub fn db(&self) -> NomadDB {
        self.db.clone()
    }

    /// Spawn a task that syncs the CachingHome's db with the on-chain event
    /// data
    pub fn sync(&self) -> JoinHandle<Result<()>> {
        let sync = self.contract_sync.clone();
        sync.spawn_home()
    }
}

#[async_trait]
impl Home for CachingHome {
    fn local_domain(&self) -> u32 {
        self.home.local_domain()
    }

    fn home_domain_hash(&self) -> H256 {
        self.home.home_domain_hash()
    }

    async fn nonces(&self, destination: u32) -> Result<u32, ChainCommunicationError> {
        self.home.nonces(destination).await
    }

    async fn dispatch(&self, message: &Message) -> Result<TxOutcome, ChainCommunicationError> {
        self.home.dispatch(message).await
    }

    async fn queue_length(&self) -> Result<U256, ChainCommunicationError> {
        self.home.queue_length().await
    }

    async fn queue_contains(&self, root: H256) -> Result<bool, ChainCommunicationError> {
        self.home.queue_contains(root).await
    }

    async fn improper_update(
        &self,
        update: &SignedUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.home.improper_update(update).await
    }

    async fn produce_update(&self) -> Result<Option<Update>, ChainCommunicationError> {
        self.home.produce_update().await
    }
}

#[async_trait]
impl HomeEvents for CachingHome {
    #[tracing::instrument(err, skip(self))]
    async fn raw_message_by_nonce(
        &self,
        destination: u32,
        nonce: u32,
    ) -> Result<Option<RawCommittedMessage>, DbError> {
        loop {
            if let Some(update) = self.db.message_by_nonce(destination, nonce)? {
                return Ok(Some(update));
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    #[tracing::instrument(err, skip(self))]
    async fn raw_message_by_leaf(
        &self,
        leaf: H256,
    ) -> Result<Option<RawCommittedMessage>, DbError> {
        loop {
            if let Some(update) = self.db.message_by_leaf(leaf)? {
                return Ok(Some(update));
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn leaf_by_tree_index(&self, tree_index: usize) -> Result<Option<H256>, DbError> {
        loop {
            if let Some(update) = self.db.leaf_by_leaf_index(tree_index as u32)? {
                return Ok(Some(update));
            }
            sleep(Duration::from_millis(500)).await;
        }
    }
}

#[async_trait]
impl Common for CachingHome {
    type Error = ChainCommunicationError;

    fn name(&self) -> &str {
        self.home.name()
    }

    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        self.home.status(txid).await
    }

    async fn updater(&self) -> Result<H256, ChainCommunicationError> {
        self.home.updater().await
    }

    async fn state(&self) -> Result<State, ChainCommunicationError> {
        self.home.state().await
    }

    async fn committed_root(&self) -> Result<H256, ChainCommunicationError> {
        self.home.committed_root().await
    }

    async fn update(&self, update: &SignedUpdate) -> Result<TxOutcome, ChainCommunicationError> {
        self.home.update(update).await
    }

    async fn double_update(
        &self,
        double: &DoubleUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.home.double_update(double).await
    }
}

#[async_trait]
impl CommonEvents for CachingHome {
    #[tracing::instrument(err, skip(self))]
    async fn signed_update_by_old_root(
        &self,
        old_root: H256,
    ) -> Result<Option<SignedUpdate>, DbError> {
        loop {
            if let Some(update) = self.db.update_by_previous_root(old_root)? {
                return Ok(Some(update));
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    #[tracing::instrument(err, skip(self))]
    async fn signed_update_by_new_root(
        &self,
        new_root: H256,
    ) -> Result<Option<SignedUpdate>, DbError> {
        loop {
            if let Some(update) = self.db.update_by_new_root(new_root)? {
                return Ok(Some(update));
            }
            sleep(Duration::from_millis(500)).await;
        }
    }
}

#[derive(Debug, Clone)]
/// Arc wrapper for HomeVariants enum
pub struct Homes(Arc<HomeVariants>);

impl std::fmt::Display for Homes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<HomeVariants> for Homes {
    fn from(homes: HomeVariants) -> Self {
        Self(Arc::new(homes))
    }
}

impl std::ops::Deref for Homes {
    type Target = Arc<HomeVariants>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Homes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Home type
#[derive(Debug)]
pub enum HomeVariants {
    /// Ethereum home contract
    Ethereum(Box<dyn Home<Error = nomad_ethereum::EthereumError>>),
    /// Substrate home object
    Substrate(Box<dyn Home<Error = nomad_substrate::SubstrateError>>),
    /// Mock home contract
    Mock(Box<MockHomeContract>),
}

impl std::fmt::Display for HomeVariants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HomeVariants::Ethereum(inner) => {
                write!(
                    f,
                    "{{ Home for {} {} }}",
                    inner.local_domain(),
                    inner.name()
                )
            }
            HomeVariants::Substrate(inner) => {
                write!(
                    f,
                    "{{ Home for {} {} }}",
                    inner.local_domain(),
                    inner.name()
                )
            }
            HomeVariants::Mock(_inner) => write!(f, "MockHome"),
        }
    }
}

impl HomeVariants {
    /// Calls checkpoint on mock variant. Should
    /// only be used during tests.
    #[doc(hidden)]
    pub fn checkpoint(&mut self) {
        if let HomeVariants::Mock(home) = self {
            home.checkpoint();
        } else {
            panic!("Home should be mock variant!");
        }
    }
}

impl<W, R> From<EthereumHome<W, R>> for Homes
where
    W: ethers::providers::Middleware + 'static,
    R: ethers::providers::Middleware + 'static,
{
    fn from(home: EthereumHome<W, R>) -> Self {
        HomeVariants::Ethereum(Box::new(home)).into()
    }
}

impl From<MockHomeContract> for Homes {
    fn from(mock_home: MockHomeContract) -> Self {
        HomeVariants::Mock(Box::new(mock_home)).into()
    }
}

#[async_trait]
impl Home for HomeVariants {
    fn local_domain(&self) -> u32 {
        match self {
            HomeVariants::Ethereum(home) => home.local_domain(),
            HomeVariants::Substrate(home) => home.local_domain(),
            HomeVariants::Mock(mock_home) => mock_home.local_domain(),
        }
    }

    fn home_domain_hash(&self) -> H256 {
        match self {
            HomeVariants::Ethereum(home) => home.home_domain_hash(),
            HomeVariants::Substrate(home) => home.home_domain_hash(),
            HomeVariants::Mock(mock_home) => mock_home.home_domain_hash(),
        }
    }

    #[instrument(level = "trace", err)]
    async fn nonces(&self, destination: u32) -> Result<u32, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.nonces(destination).await?),
            HomeVariants::Substrate(home) => Ok(home.nonces(destination).await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.nonces(destination).await?),
        }
    }

    #[instrument(level = "trace", err)]
    async fn dispatch(&self, message: &Message) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.dispatch(message).await?),
            HomeVariants::Substrate(home) => Ok(home.dispatch(message).await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.dispatch(message).await?),
        }
    }

    #[instrument(level = "trace", err)]
    async fn queue_length(&self) -> Result<U256, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.queue_length().await?),
            HomeVariants::Substrate(home) => Ok(home.queue_length().await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.queue_length().await?),
        }
    }

    async fn queue_contains(&self, root: H256) -> Result<bool, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.queue_contains(root).await?),
            HomeVariants::Substrate(home) => Ok(home.queue_contains(root).await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.queue_contains(root).await?),
        }
    }

    async fn improper_update(
        &self,
        update: &SignedUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.improper_update(update).await?),
            HomeVariants::Substrate(home) => Ok(home.improper_update(update).await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.improper_update(update).await?),
        }
    }

    #[instrument(err)]
    async fn produce_update(&self) -> Result<Option<Update>, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.produce_update().await?),
            HomeVariants::Substrate(home) => Ok(home.produce_update().await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.produce_update().await?),
        }
    }
}

#[async_trait]
impl Common for HomeVariants {
    type Error = ChainCommunicationError;

    fn name(&self) -> &str {
        match self {
            HomeVariants::Ethereum(home) => home.name(),
            HomeVariants::Substrate(home) => home.name(),
            HomeVariants::Mock(mock_home) => mock_home.name(),
        }
    }

    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.status(txid).await?),
            HomeVariants::Substrate(home) => Ok(home.status(txid).await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.status(txid).await?),
        }
    }

    async fn updater(&self) -> Result<H256, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.updater().await?),
            HomeVariants::Substrate(home) => Ok(home.updater().await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.updater().await?),
        }
    }

    async fn state(&self) -> Result<State, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.state().await?),
            HomeVariants::Substrate(home) => Ok(home.state().await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.state().await?),
        }
    }

    async fn committed_root(&self) -> Result<H256, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.committed_root().await?),
            HomeVariants::Substrate(home) => Ok(home.committed_root().await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.committed_root().await?),
        }
    }

    async fn update(&self, update: &SignedUpdate) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.update(update).await?),
            HomeVariants::Substrate(home) => Ok(home.update(update).await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.update(update).await?),
        }
    }

    async fn double_update(
        &self,
        double: &DoubleUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => Ok(home.double_update(double).await?),
            HomeVariants::Substrate(home) => Ok(home.double_update(double).await?),
            HomeVariants::Mock(mock_home) => Ok(mock_home.double_update(double).await?),
        }
    }
}
