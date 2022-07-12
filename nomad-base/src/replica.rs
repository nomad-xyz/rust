use async_trait::async_trait;
use color_eyre::eyre::Result;
use ethers::core::types::H256;
use nomad_core::{
    accumulator::NomadProof, db::DbError, ChainCommunicationError, Common, CommonEvents,
    CommonTransactions, DoubleUpdate, MessageStatus, NomadMessage, PersistedTransaction, Replica,
    ReplicaTransactions, ReplicaTxSubmitTask, SignedUpdate, State, TxContractStatus,
    TxDispatchKind, TxEventStatus, TxOutcome,
};

use crate::{NomadDB, TxSenderHandle};

use nomad_ethereum::EthereumReplica;
use nomad_test::mocks::MockReplicaContract;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::instrument::Instrumented;

use crate::{CommonIndexers, ContractSync};

/// Caching replica type
#[derive(Debug)]
pub struct CachingReplica {
    replica: Replicas,
    contract_sync: ContractSync<CommonIndexers>,
    db: NomadDB,
    tx_sender: TxSenderHandle,
}

impl std::fmt::Display for CachingReplica {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl CachingReplica {
    /// Instantiate new CachingReplica
    pub fn new(
        replica: Replicas,
        contract_sync: ContractSync<CommonIndexers>,
        db: NomadDB,
        tx_sender: TxSenderHandle,
    ) -> Self {
        Self {
            replica,
            contract_sync,
            db,
            tx_sender,
        }
    }

    /// Return handle on replica object
    pub fn replica(&self) -> Replicas {
        self.replica.clone()
    }

    /// Return handle on NomadDB
    pub fn db(&self) -> NomadDB {
        self.db.clone()
    }

    /// Spawn a task that syncs the CachingReplica's db with the on-chain event
    /// data
    pub fn sync(&self) -> Instrumented<JoinHandle<Result<()>>> {
        let sync = self.contract_sync.clone();
        sync.spawn_common()
    }
}

#[async_trait]
impl Replica for CachingReplica {
    fn local_domain(&self) -> u32 {
        self.replica.local_domain()
    }

    async fn remote_domain(&self) -> Result<u32, ChainCommunicationError> {
        self.replica.remote_domain().await
    }

    async fn message_status(&self, leaf: H256) -> Result<MessageStatus, ChainCommunicationError> {
        self.replica.message_status(leaf).await
    }

    async fn acceptable_root(&self, root: H256) -> Result<bool, ChainCommunicationError> {
        self.replica.acceptable_root(root).await
    }
}

#[async_trait]
impl ReplicaTransactions for CachingReplica {
    async fn prove(
        &self,
        _proof: &NomadProof,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        unimplemented!()
    }

    async fn process(
        &self,
        _message: &NomadMessage,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        unimplemented!()
    }
}

#[async_trait]
impl Common for CachingReplica {
    fn name(&self) -> &str {
        self.replica.name()
    }

    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        self.replica.status(txid).await
    }

    async fn updater(&self) -> Result<H256, ChainCommunicationError> {
        self.replica.updater().await
    }

    async fn state(&self) -> Result<State, ChainCommunicationError> {
        self.replica.state().await
    }

    async fn committed_root(&self) -> Result<H256, ChainCommunicationError> {
        self.replica.committed_root().await
    }
}

#[async_trait]
impl CommonTransactions for CachingReplica {
    async fn update(
        &self,
        _update: &SignedUpdate,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        unimplemented!()
    }

    async fn double_update(
        &self,
        _double: &DoubleUpdate,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        unimplemented!()
    }
}

#[async_trait]
impl CommonEvents for CachingReplica {
    #[tracing::instrument(err)]
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

    #[tracing::instrument(err)]
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
impl TxEventStatus for CachingReplica {
    async fn event_status(
        &self,
        tx: &PersistedTransaction,
    ) -> std::result::Result<TxOutcome, ChainCommunicationError> {
        self.replica.event_status(tx).await
    }
}

#[async_trait]
impl TxContractStatus for CachingReplica {
    async fn contract_status(
        &self,
        tx: &PersistedTransaction,
    ) -> std::result::Result<TxOutcome, ChainCommunicationError> {
        self.replica.contract_status(tx).await
    }
}

#[derive(Debug, Clone)]
/// Arc wrapper for ReplicaVariants enum
pub struct Replicas(Arc<ReplicaVariants>);

impl From<ReplicaVariants> for Replicas {
    fn from(replicas: ReplicaVariants) -> Self {
        Self(Arc::new(replicas))
    }
}

impl std::ops::Deref for Replicas {
    type Target = Arc<ReplicaVariants>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Replicas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Replica type
#[derive(Debug)]
pub enum ReplicaVariants {
    /// Ethereum replica contract
    Ethereum(Box<dyn ReplicaTxSubmitTask>),
    /// Mock replica contract
    Mock(Box<MockReplicaContract>),
    /// Other replica variant
    Other(Box<dyn ReplicaTxSubmitTask>),
}

impl ReplicaVariants {
    /// Calls checkpoint on mock variant. Should
    /// only be used during tests.
    #[doc(hidden)]
    pub fn checkpoint(&mut self) {
        if let ReplicaVariants::Mock(replica) = self {
            replica.checkpoint();
        } else {
            panic!("Replica should be mock variant!");
        }
    }
}

impl<W, R> From<EthereumReplica<W, R>> for Replicas
where
    W: ethers::providers::Middleware + 'static,
    R: ethers::providers::Middleware + 'static,
{
    fn from(replica: EthereumReplica<W, R>) -> Self {
        ReplicaVariants::Ethereum(Box::new(replica)).into()
    }
}

impl From<MockReplicaContract> for Replicas {
    fn from(mock_replica: MockReplicaContract) -> Self {
        ReplicaVariants::Mock(Box::new(mock_replica)).into()
    }
}

impl From<Box<dyn ReplicaTxSubmitTask>> for Replicas {
    fn from(replica: Box<dyn ReplicaTxSubmitTask>) -> Self {
        ReplicaVariants::Other(replica).into()
    }
}

#[async_trait]
impl Replica for ReplicaVariants {
    fn local_domain(&self) -> u32 {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.local_domain(),
            ReplicaVariants::Mock(mock_replica) => mock_replica.local_domain(),
            ReplicaVariants::Other(replica) => replica.local_domain(),
        }
    }

    async fn remote_domain(&self) -> Result<u32, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.remote_domain().await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.remote_domain().await,
            ReplicaVariants::Other(replica) => replica.remote_domain().await,
        }
    }

    async fn message_status(&self, leaf: H256) -> Result<MessageStatus, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.message_status(leaf).await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.message_status(leaf).await,
            ReplicaVariants::Other(replica) => replica.message_status(leaf).await,
        }
    }

    async fn acceptable_root(&self, root: H256) -> Result<bool, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.acceptable_root(root).await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.acceptable_root(root).await,
            ReplicaVariants::Other(replica) => replica.acceptable_root(root).await,
        }
    }
}

#[async_trait]
impl Common for ReplicaVariants {
    fn name(&self) -> &str {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.name(),
            ReplicaVariants::Mock(mock_replica) => mock_replica.name(),
            ReplicaVariants::Other(replica) => replica.name(),
        }
    }

    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.status(txid).await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.status(txid).await,
            ReplicaVariants::Other(replica) => replica.status(txid).await,
        }
    }

    async fn updater(&self) -> Result<H256, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.updater().await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.updater().await,
            ReplicaVariants::Other(replica) => replica.updater().await,
        }
    }

    async fn state(&self) -> Result<State, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.state().await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.state().await,
            ReplicaVariants::Other(replica) => replica.state().await,
        }
    }

    async fn committed_root(&self) -> Result<H256, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.committed_root().await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.committed_root().await,
            ReplicaVariants::Other(replica) => replica.committed_root().await,
        }
    }
}

#[async_trait]
impl TxEventStatus for ReplicaVariants {
    async fn event_status(
        &self,
        _tx: &PersistedTransaction,
    ) -> std::result::Result<TxOutcome, ChainCommunicationError> {
        unimplemented!()
    }
}

#[async_trait]
impl TxContractStatus for ReplicaVariants {
    async fn contract_status(
        &self,
        _tx: &PersistedTransaction,
    ) -> std::result::Result<TxOutcome, ChainCommunicationError> {
        unimplemented!()
    }
}
