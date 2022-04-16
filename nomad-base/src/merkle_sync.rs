use color_eyre::Result;
use nomad_core::{
    accumulator::incremental::IncrementalMerkle, db::DbError, ChainCommunicationError,
};
use std::time::Duration;
use tokio::{task::JoinHandle, time::sleep};
use tracing::{debug, error, info, info_span, instrument::Instrumented, Instrument};

use crate::NomadDB;

/// Self-syncing light merkle tree. Polls for new messages and updates tree.
#[derive(Debug, Clone)]
pub struct IncrementalMerkleSync {
    /// Light merkle tree
    pub tree: IncrementalMerkle,
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
    pub fn new(db: NomadDB) -> Self {
        Self {
            tree: Default::default(),
            db,
        }
    }

    /// Instantiate new IncrementalMerkleSync from DB
    pub fn from_disk(db: NomadDB) -> Self {
        let mut tree = IncrementalMerkle::default();
        if let Some(root) = db.retrieve_latest_root().expect("db error") {
            for i in 0.. {
                match db.leaf_by_leaf_index(i) {
                    Ok(Some(leaf)) => {
                        debug!(leaf_index = i, "Ingesting leaf from_disk");
                        tree.ingest(leaf);
                        if tree.root() == root {
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
            info!(target_latest_root = ?root, root = ?tree.root(), "Reloaded IncrementalMerkleSync from_disk");
        }

        Self { tree, db }
    }

    /// Start syncing merkle tree with DB
    pub fn sync(&self) -> Instrumented<JoinHandle<Result<()>>> {
        let mut tree = self.tree.clone();
        let db = self.db.clone();

        let span = info_span!("IncrementalMerkleSync");
        tokio::spawn(async move {
            loop {
                let tree_size = tree.count();

                info!("Waiting for leaf at index {}...", tree_size);
                let leaf = db.wait_for_leaf(tree_size as u32).await?;

                info!(
                    index = tree_size,
                    leaf = ?leaf,
                    "Ingesting leaf at index {}. Leaf: {}.",
                    tree_size,
                    leaf
                );
                tree.ingest(leaf);
                sleep(Duration::from_secs(5)).await;
            }
        })
        .instrument(span)
    }
}
