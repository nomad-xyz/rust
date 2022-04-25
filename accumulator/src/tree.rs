use crate::{full::MerkleTree, IngestionError, LightMerkle, Merkle, Proof, ProvingError};
use ethers::{core::types::H256, prelude::U256};

/// A simplified interface for a full sparse merkle tree
#[derive(Debug, PartialEq)]
pub struct Tree<const N: usize> {
    count: usize,
    tree: Box<MerkleTree>,
}

impl<const N: usize> Default for Tree<N> {
    fn default() -> Self {
        Self::from_leaves(&[])
    }
}

impl<const N: usize> Merkle for Tree<N> {
    type Proof = Proof<N>;

    /// Return the maximum number of leaves in this tree
    fn max_elements() -> U256 {
        crate::utils::max_leaves(N)
    }

    fn count(&self) -> usize {
        self.count
    }

    fn root(&self) -> H256 {
        self.tree.hash()
    }

    fn depth(&self) -> usize {
        N
    }

    fn ingest(&mut self, element: H256) -> Result<H256, IngestionError> {
        self.count += 1;
        self.tree.push_leaf(element, N)?;
        Ok(self.tree.hash())
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

    /// Return the leaf at `index` and a Merkle proof of its inclusion.
    ///
    /// The Merkle proof is in "bottom-up" order, starting with a leaf node
    /// and moving up the tree. Its length will be exactly equal to `depth`.
    pub fn prove(&self, index: usize) -> Result<Proof<N>, ProvingError> {
        if index > 2usize.pow(N.try_into().unwrap()) - 1 {
            return Err(ProvingError::IndexTooHigh(index));
        }

        let count = self.count();
        if index >= count {
            return Err(ProvingError::ZeroProof { index, count });
        }

        let (leaf, nodes) = self.tree.generate_proof(index, N);
        debug_assert_eq!(nodes.len(), N);
        let mut path = [H256::default(); N];
        path.copy_from_slice(&nodes[..N]);
        Ok(Proof { leaf, index, path })
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
