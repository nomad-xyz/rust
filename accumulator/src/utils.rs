use sha3::{Digest, Keccak256};

use ethers::core::types::{H256, U256};

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

/// Max number of leaves in a tree
pub(crate) fn max_leaves(n: usize) -> U256 {
    U256::from(2).pow(n.into()) - 1
}
