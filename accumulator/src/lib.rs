//! A set of accumulator-related tooling for Nomad development. This crate contains a full incremental merkle tree, as well as a lightweight-

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

/// A full incremental merkle. Suitable for running off-chain.
pub mod full;
/// A lightweight incremental merkle, suitable for running on-chain. Stores O
/// (1) data
pub mod light;
/// Merkle Proof struct
pub mod proof;

/// ...
pub mod tree;

#[cfg(target_arch = "wasm32")]
/// Wasm bindings for common operations
pub mod wasm;

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", global_allocator)]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use ethers::core::types::H256;
use lazy_static::lazy_static;
use sha3::{Digest, Keccak256};

/// Tree depth
pub const TREE_DEPTH: usize = 32;
/// A Nomad protocol standard-depth tree
pub type NomadTree = tree::Tree<TREE_DEPTH>;
/// An incremental Nomad protocol standard-depth tree
pub type NomadLightMerkle = light::LightMerkle<TREE_DEPTH>;
/// A Nomad protocol standard-depth proof
pub type NomadProof = proof::Proof<TREE_DEPTH>;

const EMPTY_SLICE: &[H256] = &[];

pub use full::*;
pub use light::*;
pub use proof::*;
pub use tree::*;

/// Return the keccak256 digest of the preimage
pub fn hash(preimage: impl AsRef<[u8]>) -> H256 {
    H256::from_slice(Keccak256::digest(preimage.as_ref()).as_slice())
}

/// Return the keccak256 disgest of the concatenation of the arguments
pub fn hash_concat(left: impl AsRef<[u8]>, right: impl AsRef<[u8]>) -> H256 {
    H256::from_slice(
        Keccak256::new()
            .chain(left.as_ref())
            .chain(right.as_ref())
            .finalize()
            .as_slice(),
    )
}

lazy_static! {
    /// A cache of the zero hashes for each layer of the tree.
    pub static ref ZERO_HASHES: [H256; TREE_DEPTH + 1] = {
        let mut hashes = [H256::zero(); TREE_DEPTH + 1];
        for i in 0..TREE_DEPTH {
            hashes[i + 1] = hash_concat(hashes[i], hashes[i]);
        }
        hashes
    };
}
