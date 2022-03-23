use crate::{full::MerkleTree, merkle_root_from_branch, LightMerkle, MerkleTreeError, Proof};
use ethers::core::types::H256;

/// A simplified interface for a full sparse merkle tree
#[derive(Debug, PartialEq)]
pub struct Tree<const N: usize> {
    count: usize,
    tree: Box<MerkleTree>,
}

/// Tree Errors
#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum TreeError {
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
    /// Bubbled up from underlying
    #[error(transparent)]
    MerkleTreeError(#[from] MerkleTreeError),
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

impl<const N: usize> Default for Tree<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Tree<N> {
    /// Instantiate a new tree with a known depth and a starting leaf-set
    pub fn from_leaves(leaves: &[H256]) -> Self {
        Self {
            count: leaves.len(),
            tree: Box::new(MerkleTree::create(leaves, N)),
        }
    }

    /// Calculate the initital root of a tree of this depth
    pub fn initial_root() -> H256 {
        LightMerkle::<N>::default().root()
    }

    /// Instantiate a new tree with a known depth and no leaves
    pub fn new() -> Self {
        Self::from_leaves(&[])
    }

    /// Push a leaf to the tree. Appends it to the first unoccupied slot
    ///
    /// This will fail if the underlying tree is full.
    pub fn ingest(&mut self, element: H256) -> Result<H256, TreeError> {
        self.count += 1;
        self.tree.push_leaf(element, N)?;
        Ok(self.tree.hash())
    }

    /// Retrieve the root hash of this Merkle tree.
    pub fn root(&self) -> H256 {
        self.tree.hash()
    }

    /// Get the tree's depth.
    pub fn depth(&self) -> usize {
        N
    }

    /// Return the leaf at `index` and a Merkle proof of its inclusion.
    ///
    /// The Merkle proof is in "bottom-up" order, starting with a leaf node
    /// and moving up the tree. Its length will be exactly equal to `depth`.
    pub fn prove(&self, index: usize) -> Result<Proof<N>, TreeError> {
        if index > 2usize.pow(N.try_into().unwrap()) - 1 {
            return Err(TreeError::IndexTooHigh(index));
        }

        let count = self.count();
        if index >= count {
            return Err(TreeError::ZeroProof { index, count });
        }

        let (leaf, nodes) = self.tree.generate_proof(index, N);
        debug_assert_eq!(nodes.len(), N);
        let mut path = [H256::default(); N];
        path.copy_from_slice(&nodes[..N]);
        Ok(Proof { leaf, index, path })
    }

    /// Verify a proof against this tree's root.
    #[allow(dead_code)]
    pub fn verify(&self, proof: &Proof<N>) -> Result<(), TreeError> {
        let actual = merkle_root_from_branch(proof.leaf, &proof.path, N, proof.index);
        let expected = self.root();
        if expected == actual {
            Ok(())
        } else {
            Err(TreeError::VerificationFailed { expected, actual })
        }
    }

    /// Get the tree's leaf count.
    pub fn count(&self) -> usize {
        self.count
    }
}

impl<T, const N: usize> From<T> for Tree<N>
where
    T: AsRef<[H256]>,
{
    fn from(t: T) -> Self {
        Self::from_leaves(t.as_ref())
    }
}

impl<const N: usize> std::iter::FromIterator<H256> for Tree<N> {
    /// Will panic if the tree fills
    fn from_iter<I: IntoIterator<Item = H256>>(iter: I) -> Self {
        let mut prover = Self::default();
        prover.extend(iter);
        prover
    }
}

impl<const N: usize> std::iter::Extend<H256> for Tree<N> {
    /// Will panic if the tree fills
    fn extend<I: IntoIterator<Item = H256>>(&mut self, iter: I) {
        for i in iter {
            self.ingest(i).expect("!tree full");
        }
    }
}
