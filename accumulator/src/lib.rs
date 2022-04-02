#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

/// A full incremental merkle. Suitable for running off-chain.
mod full;

/// Hashing utils
pub mod utils;

/// Common error types for the merkle trees.
pub mod error;

/// A lightweight incremental merkle, suitable for running on-chain. Stores O
/// (1) data
pub mod light;
/// Merkle Proof struct
pub mod proof;

/// A full incremental merkle tree. Suitable for proving.
pub mod tree;

#[cfg(target_arch = "wasm32")]
/// Wasm bindings for common operations
pub mod wasm;

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", global_allocator)]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use ethers::core::types::H256;
use lazy_static::lazy_static;

/// Tree depth
pub const TREE_DEPTH: usize = 32;
/// A Nomad protocol standard-depth tree
pub type NomadTree = tree::Tree<TREE_DEPTH>;
/// An incremental Nomad protocol standard-depth tree
pub type NomadLightMerkle = light::LightMerkle<TREE_DEPTH>;
/// A Nomad protocol standard-depth proof
pub type NomadProof = proof::Proof<TREE_DEPTH>;

const EMPTY_SLICE: &[H256] = &[];

pub use error::*;
use full::*;
pub use light::*;
pub use proof::*;
pub use tree::*;

pub use utils::*;

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
