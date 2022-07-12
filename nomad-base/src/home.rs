use crate::{ContractSync, HomeIndexers, NomadDB, TxManager};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use ethers::core::types::{H256, U256};
use nomad_core::{
    db::DbError, ChainCommunicationError, Common, CommonEvents, CommonTxHandling,
    CommonTxSubmission, DoubleUpdate, Home, HomeEvents, HomeTxHandling, HomeTxSubmission, Message,
    NomadMethod, PersistedTransaction, RawCommittedMessage, SignedUpdate, State, TxContractStatus,
    TxDispatchKind, TxEventStatus, TxOutcome, Update,
};
use nomad_ethereum::EthereumHome;
use nomad_test::mocks::MockHomeContract;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{instrument, instrument::Instrumented};

/// Caching replica type
#[derive(Debug)]
pub struct CachingHome {
    home: Homes,
    contract_sync: ContractSync<HomeIndexers>,
    db: NomadDB,
    tx_manager: TxManager,
}

impl std::fmt::Display for CachingHome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl CachingHome {
    /// Instantiate new CachingHome
    pub fn new(
        home: Homes,
        contract_sync: ContractSync<HomeIndexers>,
        db: NomadDB,
        tx_manager: TxManager,
    ) -> Self {
        Self {
            home,
            contract_sync,
            db,
            tx_manager,
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
    pub fn sync(&self) -> Instrumented<JoinHandle<Result<()>>> {
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

    async fn queue_length(&self) -> Result<U256, ChainCommunicationError> {
        self.home.queue_length().await
    }

    async fn queue_contains(&self, root: H256) -> Result<bool, ChainCommunicationError> {
        self.home.queue_contains(root).await
    }

    async fn produce_update(&self) -> Result<Option<Update>, ChainCommunicationError> {
        self.home.produce_update().await
    }
}

#[async_trait]
impl HomeTxHandling for CachingHome {
    async fn dispatch(
        &self,
        message: &Message,
        dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.tx_manager
            .submit_transaction(
                NomadMethod::Dispatch(message.to_owned()).into(),
                dispatch_kind,
            )
            .await
    }

    async fn improper_update(
        &self,
        update: &SignedUpdate,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.home.improper_update(update).await
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
impl TxEventStatus for CachingHome {
    async fn event_status(
        &self,
        tx: &PersistedTransaction,
    ) -> std::result::Result<TxOutcome, ChainCommunicationError> {
        self.home.event_status(tx).await
    }
}

#[async_trait]
impl Common for CachingHome {
    fn name(&self) -> &str {
        self.home.name()
    }

    async fn status(
        &self,
        txid: H256
    ) -> Result<Option<TxOutcome>, ChainCommunicationError> {
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
}

#[async_trait]
impl CommonTxHandling for CachingHome {
    async fn status(
        &self,
        txid: H256,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        self.home.status(txid).await
    }

    async fn update(
        &self,
        update: &SignedUpdate,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.home.update(update).await
    }

    async fn double_update(
        &self,
        double: &DoubleUpdate,
        _dispatch_kind: TxDispatchKind,
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

#[async_trait]
impl TxContractStatus for CachingHome {
    async fn contract_status(
        &self,
        tx: &PersistedTransaction,
    ) -> std::result::Result<TxOutcome, ChainCommunicationError> {
        self.home.contract_status(tx).await
    }
}

#[derive(Debug, Clone)]
/// Arc wrapper for HomeVariants enum
pub struct Homes(Arc<HomeVariants>);

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
    Ethereum(Box<dyn HomeTxSubmission>),
    /// Mock home contract
    Mock(Box<MockHomeContract>),
    /// Other home variant
    Other(Box<dyn HomeTxSubmission>),
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

impl From<Box<dyn HomeTxSubmission>> for Homes {
    fn from(home: Box<dyn HomeTxSubmission>) -> Self {
        HomeVariants::Other(home).into()
    }
}

#[async_trait]
impl Home for HomeVariants {
    fn local_domain(&self) -> u32 {
        match self {
            HomeVariants::Ethereum(home) => home.local_domain(),
            HomeVariants::Mock(mock_home) => mock_home.local_domain(),
            HomeVariants::Other(home) => home.local_domain(),
        }
    }

    fn home_domain_hash(&self) -> H256 {
        match self {
            HomeVariants::Ethereum(home) => home.home_domain_hash(),
            HomeVariants::Mock(mock_home) => mock_home.home_domain_hash(),
            HomeVariants::Other(home) => home.home_domain_hash(),
        }
    }

    #[instrument(level = "trace", err)]
    async fn nonces(&self, destination: u32) -> Result<u32, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.nonces(destination).await,
            HomeVariants::Mock(mock_home) => mock_home.nonces(destination).await,
            HomeVariants::Other(home) => home.nonces(destination).await,
        }
    }

    #[instrument(level = "trace", err)]
    async fn queue_length(&self) -> Result<U256, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.queue_length().await,
            HomeVariants::Mock(mock_home) => mock_home.queue_length().await,
            HomeVariants::Other(home) => home.queue_length().await,
        }
    }

    async fn queue_contains(&self, root: H256) -> Result<bool, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.queue_contains(root).await,
            HomeVariants::Mock(mock_home) => mock_home.queue_contains(root).await,
            HomeVariants::Other(home) => home.queue_contains(root).await,
        }
    }

    #[instrument(err)]
    async fn produce_update(&self) -> Result<Option<Update>, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.produce_update().await,
            HomeVariants::Mock(mock_home) => mock_home.produce_update().await,
            HomeVariants::Other(home) => home.produce_update().await,
        }
    }
}

#[async_trait]
impl HomeTxSubmission for HomeVariants {
    #[instrument(level = "trace", err)]
    async fn dispatch(&self, message: &Message) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.dispatch(message).await,
            HomeVariants::Mock(mock_home) => mock_home.dispatch(message).await,
            HomeVariants::Other(home) => home.dispatch(message).await,
        }
    }

    async fn improper_update(
        &self,
        update: &SignedUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.improper_update(update).await,
            HomeVariants::Mock(mock_home) => mock_home.improper_update(update).await,
            HomeVariants::Other(home) => home.improper_update(update).await,
        }
    }
}

#[async_trait]
impl Common for HomeVariants {
    fn name(&self) -> &str {
        match self {
            HomeVariants::Ethereum(home) => home.name(),
            HomeVariants::Mock(mock_home) => mock_home.name(),
            HomeVariants::Other(home) => home.name(),
        }
    }

    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.status(txid).await,
            HomeVariants::Mock(mock_home) => mock_home.status(txid).await,
            HomeVariants::Other(home) => home.status(txid).await,
        }
    }

    async fn updater(&self) -> Result<H256, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.updater().await,
            HomeVariants::Mock(mock_home) => mock_home.updater().await,
            HomeVariants::Other(home) => home.updater().await,
        }
    }

    async fn state(&self) -> Result<State, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.state().await,
            HomeVariants::Mock(mock_home) => mock_home.state().await,
            HomeVariants::Other(home) => home.state().await,
        }
    }

    async fn committed_root(&self) -> Result<H256, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.committed_root().await,
            HomeVariants::Mock(mock_home) => mock_home.committed_root().await,
            HomeVariants::Other(home) => home.committed_root().await,
        }
    }
}

#[async_trait]
impl CommonTxSubmission for HomeVariants {
    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.status(txid).await,
            HomeVariants::Mock(mock_home) => mock_home.status(txid).await,
            HomeVariants::Other(home) => home.status(txid).await,
        }
    }

    async fn update(&self, update: &SignedUpdate) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.update(update).await,
            HomeVariants::Mock(mock_home) => mock_home.update(update).await,
            HomeVariants::Other(home) => home.update(update).await,
        }
    }

    async fn double_update(
        &self,
        double: &DoubleUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.double_update(double).await,
            HomeVariants::Mock(mock_home) => mock_home.double_update(double).await,
            HomeVariants::Other(home) => home.double_update(double).await,
        }
    }
}

#[async_trait]
impl TxEventStatus for HomeVariants {
    async fn event_status(
        &self,
        tx: &PersistedTransaction,
    ) -> std::result::Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.event_status(tx).await,
            _ => unimplemented!(),
        }
    }
}

#[async_trait]
impl TxContractStatus for HomeVariants {
    async fn contract_status(
        &self,
        tx: &PersistedTransaction,
    ) -> std::result::Result<TxOutcome, ChainCommunicationError> {
        match self {
            HomeVariants::Ethereum(home) => home.contract_status(tx).await,
            _ => unimplemented!(),
        }
    }
}
