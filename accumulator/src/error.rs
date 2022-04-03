use ethers::prelude::H256;
use thiserror;

/// Tree Errors
#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum ProvingError {
    /// Index is above tree max size
    #[error("Requested proof for index above u32::MAX: {0}")]
    IndexTooHigh(usize),
    /// Requested proof for a zero element
    #[error("Requested proof for a zero element. Requested: {index}. Tree has: {count}")]
    ZeroProof {
        /// The index requested
        index: usize,
        /// The number of leaves
        count: usize,
    },
}

/// Tree Errors
#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum VerifyingError {
    /// Failed proof verification
    #[error("Proof verification failed. Root is {expected}, produced is {actual}")]
    #[allow(dead_code)]
    VerificationFailed {
        /// The expected root (this tree's current root)
        expected: H256,
        /// The root produced by branch evaluation
        actual: H256,
    },
}

/// Error type for merkle tree ops.
#[derive(Debug, PartialEq, Clone, Copy, thiserror::Error)]
pub enum IngestionError {
    /// Trying to push in a leaf
    #[error("Trying to push in a leaf")]
    LeafReached,
    /// No more space in the MerkleTree
    #[error("No more space in the MerkleTree")]
    MerkleTreeFull,
    /// MerkleTree is invalid
    #[error("MerkleTree is invalid")]
    Invalid,
    /// Incorrect Depth provided
    #[error("Incorrect Depth provided")]
    DepthTooSmall,
}
