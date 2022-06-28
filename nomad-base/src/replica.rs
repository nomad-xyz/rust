use async_trait::async_trait;
use color_eyre::eyre::Result;
use ethers::core::types::H256;
use nomad_core::{
    accumulator::NomadProof, db::DbError, ChainCommunicationError, Common, CommonEvents,
    CommonTxHandling, CommonTxSubmission, DoubleUpdate, MessageStatus, NomadMessage,
    PersistedTransaction, Replica, ReplicaTxHandling, ReplicaTxSubmission, SignedUpdate, State,
    TxDispatchKind, TxForwarder, TxOutcome,
};

use crate::NomadDB;

use nomad_ethereum::EthereumReplica;
use nomad_test::mocks::MockReplicaContract;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{instrument, instrument::Instrumented};

use crate::{CommonIndexers, ContractSync};

/// Caching replica type
#[derive(Debug)]
pub struct CachingReplica {
    replica: Replicas,
    contract_sync: ContractSync<CommonIndexers>,
    db: NomadDB,
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
    ) -> Self {
        Self {
            replica,
            contract_sync,
            db,
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
impl ReplicaTxHandling for CachingReplica {
    async fn prove(
        &self,
        proof: &NomadProof,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.replica.prove(proof).await
    }

    async fn process(
        &self,
        message: &NomadMessage,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.replica.process(message).await
    }
}

#[async_trait]
impl Common for CachingReplica {
    fn name(&self) -> &str {
        self.replica.name()
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
impl CommonTxHandling for CachingReplica {
    async fn status(
        &self,
        txid: H256,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        self.replica.status(txid).await
    }

    async fn update(
        &self,
        update: &SignedUpdate,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.replica.update(update).await
    }

    async fn double_update(
        &self,
        double: &DoubleUpdate,
        _dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.replica.double_update(double).await
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
impl TxForwarder for CachingReplica {
    async fn forward(
        &self,
        tx: PersistedTransaction,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.replica.forward(tx).await
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
    Ethereum(Box<dyn ReplicaTxSubmission>),
    /// Mock replica contract
    Mock(Box<MockReplicaContract>),
    /// Other replica variant
    Other(Box<dyn ReplicaTxSubmission>),
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

impl From<Box<dyn ReplicaTxSubmission>> for Replicas {
    fn from(replica: Box<dyn ReplicaTxSubmission>) -> Self {
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
impl ReplicaTxSubmission for ReplicaVariants {
    async fn prove(&self, proof: &NomadProof) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.prove(proof).await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.prove(proof).await,
            ReplicaVariants::Other(replica) => replica.prove(proof).await,
        }
    }

    async fn process(&self, message: &NomadMessage) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.process(message).await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.process(message).await,
            ReplicaVariants::Other(replica) => replica.process(message).await,
        }
    }

    async fn prove_and_process(
        &self,
        message: &NomadMessage,
        proof: &NomadProof,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.prove_and_process(message, proof).await,
            ReplicaVariants::Mock(mock_replica) => {
                mock_replica.prove_and_process(message, proof).await
            }
            ReplicaVariants::Other(replica) => replica.prove_and_process(message, proof).await,
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
impl CommonTxSubmission for ReplicaVariants {
    async fn status(&self, txid: H256) -> Result<Option<TxOutcome>, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.status(txid).await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.status(txid).await,
            ReplicaVariants::Other(replica) => replica.status(txid).await,
        }
    }

    #[instrument(fields(update = %update.update))]
    async fn update(&self, update: &SignedUpdate) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.update(update).await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.update(update).await,
            ReplicaVariants::Other(replica) => replica.update(update).await,
        }
    }

    async fn double_update(
        &self,
        double: &DoubleUpdate,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(replica) => replica.double_update(double).await,
            ReplicaVariants::Mock(mock_replica) => mock_replica.double_update(double).await,
            ReplicaVariants::Other(replica) => replica.double_update(double).await,
        }
    }
}

#[async_trait]
impl TxForwarder for ReplicaVariants {
    async fn forward(
        &self,
        tx: PersistedTransaction,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        match self {
            ReplicaVariants::Ethereum(home) => home.forward(tx).await,
            _ => unimplemented!(),
        }
    }
}
