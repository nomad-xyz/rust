use ethers::core::types::H256;
use sha3::{Digest, Keccak256};

/// Computes hash of home domain concatenated with "NOMAD"
pub fn home_domain_hash(home_domain: u32) -> H256 {
    H256::from_slice(
        Keccak256::new()
            .chain(home_domain.to_be_bytes())
            .chain("NOMAD".as_bytes())
            .finalize()
            .as_slice(),
    )
}

/// Destination and destination-specific nonce combined in single field (
/// (destination << 32) & nonce)
pub fn destination_and_nonce(destination: u32, nonce: u32) -> u64 {
    assert!(destination < u32::MAX);
    assert!(nonce < u32::MAX);
    ((destination as u64) << 32) | nonce as u64
}
