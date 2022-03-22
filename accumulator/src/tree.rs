use crate::{full::MerkleTree, MerkleTreeError};
use ethers::core::types::H256;

/// A simplified interface for a full sparse merkle tree
#[derive(Debug, PartialEq)]
pub struct Tree {
    depth: usize,
    count: usize,
    tree: Box<MerkleTree>,
}

impl Tree {
    /// Instantiate a new tree with a known depth and a starting leaf-set
    pub fn from_leaves(leaves: &[H256], depth: usize) -> Self {
        Self {
            depth,
            count: leaves.len(),
            tree: Box::new(MerkleTree::create(leaves, depth as usize)),
        }
    }

    /// Instantiate a new tree with a known depth and no leaves
    pub fn new(depth: usize) -> Self {
        Self::from_leaves(&[], depth)
    }

    /// Push an element into the MerkleTree.
    pub fn push_leaf(&mut self, leaf: H256) -> Result<(), MerkleTreeError> {
        self.count += 1;
        self.tree.push_leaf(leaf, self.depth)
    }

    /// Retrieve the root hash of this Merkle tree.
    pub fn root(&self) -> H256 {
        self.tree.hash()
    }

    /// Get the tree's depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Return the leaf at `index` and a Merkle proof of its inclusion.
    ///
    /// The Merkle proof is in "bottom-up" order, starting with a leaf node
    /// and moving up the tree. Its length will be exactly equal to `depth`.
    pub fn generate_proof(&self, index: usize) -> (H256, Vec<H256>) {
        self.tree.generate_proof(index, self.depth)
    }

    /// Get the tree's leaf count.
    pub fn count(&self) -> usize {
        self.count
    }
}
