use color_eyre::Result;
use ethers::types::H256;
use nomad_core::{
    accumulator::incremental::IncrementalMerkle, db::DbError, ChainCommunicationError,
    RawCommittedMessage,
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};
use tracing::{debug, error, info, info_span, instrument::Instrumented, Instrument};

use crate::NomadDB;

/// Incremental merkle tree that tracks current committed root
#[derive(Debug, Clone, Default)]
pub struct NomadIncrementalMerkle {
    /// Light merkle tree
    pub tree: IncrementalMerkle,
    /// Last committed root of tree (last signed new_root)
    pub last_committed_root: H256,
}

impl NomadIncrementalMerkle {
    pub fn new(tree: IncrementalMerkle, last_committed_root: H256) -> Self {
        Self {
            tree,
            last_committed_root,
        }
    }

    /// Fetch the current root of the tree
    pub fn root(&self) -> H256 {
        self.tree.root()
    }

    /// Fetch current size of tree
    pub fn count(&self) -> usize {
        self.tree.count()
    }

    /// Fetch last committed root (last signed new_root)
    pub fn last_committed_root(&self) -> H256 {
        self.last_committed_root
    }

    /// Ingest message to tree and update last committed root
    pub fn ingest_message(&mut self, message: &RawCommittedMessage) {
        self.tree.ingest(message.leaf());
        self.last_committed_root = message.committed_root;
    }
}

/// Self-syncing light merkle tree. Polls for new messages and updates tree.
#[derive(Debug, Clone)]
pub struct IncrementalMerkleSync {
    /// Self syncing merkle tree with tracked last committed root
    pub merkle: Arc<RwLock<NomadIncrementalMerkle>>,
    /// DB with home name key prefix
    pub db: NomadDB,
}

/// IncrementalMerkleSync errors
#[derive(Debug, thiserror::Error)]
pub enum IncrementalMerkleSyncError {
    /// IncrementalMerkleSync receives ChainCommunicationError from chain API
    #[error(transparent)]
    ChainCommunicationError(#[from] ChainCommunicationError),
    /// DB Error
    #[error("{0}")]
    DbError(#[from] DbError),
}

impl IncrementalMerkleSync {
    /// Instantiate new IncrementalMerkleSync
    pub fn default(db: NomadDB) -> Self {
        Self {
            merkle: Arc::new(RwLock::new(Default::default())),
            db,
        }
    }

    /// Instantiate new IncrementalMerkleSync from DB
    pub fn from_disk(db: NomadDB) -> Self {
        let mut tree = IncrementalMerkle::default();
        let mut last_committed_root = H256::default();

        if let Some(latest_root) = db.retrieve_latest_root().expect("db error") {
            for i in 0.. {
                match db.message_by_leaf_index(i) {
                    Ok(Some(message)) => {
                        debug!(
                            leaf_index = i,
                            last_committed_root = ?last_committed_root,
                            "Ingesting leaf from_disk"
                        );

                        tree.ingest(message.leaf());
                        last_committed_root = message.committed_root;

                        if tree.root() == latest_root {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        error!(error = %e, "Error in IncrementalMerkleSync::from_disk");
                        panic!("Error in IncrementalMerkleSync::from_disk");
                    }
                }
            }

            info!(target_latest_root = ?latest_root, last_committed_root = ?last_committed_root, root = ?tree.root(), "Reloaded IncrementalMerkleSync from_disk");
        }

        Self {
            merkle: Arc::new(RwLock::new(NomadIncrementalMerkle::new(
                tree,
                last_committed_root,
            ))),
            db,
        }
    }

    /// Fetch the current root of the tree
    pub async fn root(&self) -> H256 {
        self.merkle.read().await.root()
    }

    pub async fn last_committed_root(&self) -> H256 {
        self.merkle.read().await.last_committed_root()
    }

    /// Start syncing merkle tree with DB
    pub fn sync(&self) -> Instrumented<JoinHandle<Result<()>>> {
        let merkle = self.merkle.clone();
        let db = self.db.clone();

        let span = info_span!("IncrementalMerkleSync");
        tokio::spawn(async move {
            loop {
                let tree_size = merkle.read().await.count();

                info!("Waiting for leaf at index {}...", tree_size);
                let message = db.wait_for_message(tree_size as u32).await?;

                info!(
                    index = tree_size,
                    leaf = ?message.leaf(),
                    "Ingesting leaf at index {}. Leaf: {}.",
                    tree_size,
                    message.leaf(),
                );

                merkle.write().await.ingest_message(&message);
                sleep(Duration::from_secs(3)).await;
            }
        })
        .instrument(span)
    }
}
