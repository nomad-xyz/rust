use nomad_core::accumulator::{self, arrays, TREE_DEPTH};
use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};

/// Substrate-specific Nomad states. Does not include an uninitialized state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NomadState {
    /// Contract is active
    Active,
    /// Contract has failed
    Failed,
}

impl Default for NomadState {
    fn default() -> Self {
        Self::Active
    }
}

/// Wrapper for accomodating oddities of scale-value encoding of H256 primitives.
/// Need wrapper type to match the shape of the scale encoded value.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct H256Wrapper([H256; 1]);

impl std::ops::Deref for H256Wrapper {
    type Target = [u8; 32];
    fn deref(&self) -> &Self::Target {
        &self.0[0].0
    }
}

impl From<H256Wrapper> for ethers_core::types::H256 {
    fn from(wrapper: H256Wrapper) -> Self {
        wrapper.into()
    }
}

/// Wrapper for accomodating oddities of scale-value encoding of H160 primitives.
/// Need wrapper type to match the shape of the scale encoded value.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct H160Wrapper([H160; 1]);

impl std::ops::Deref for H160Wrapper {
    type Target = [u8; 20];
    fn deref(&self) -> &Self::Target {
        &self.0[0].0
    }
}

impl From<H160Wrapper> for ethers_core::types::H160 {
    fn from(wrapper: H160Wrapper) -> Self {
        wrapper.into()
    }
}

/// NomadBase struct stored in Substrate home
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct NomadBase {
    /// State
    pub state: NomadState,
    /// Local domain
    pub local_domain: u32,
    /// Committed root
    pub committed_root: H256Wrapper,
    /// Updater
    pub updater: H160Wrapper,
}

/// An incremental merkle tree wrapper that uses the H256Wrapper type.
/// Accomodates oddities of the scale value encoding of H256.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NomadLightMerkleWrapper {
    #[serde(with = "arrays")]
    branch: [H256Wrapper; TREE_DEPTH],
    count: usize,
}

impl From<NomadLightMerkleWrapper> for accumulator::NomadLightMerkle {
    fn from(wrapper: NomadLightMerkleWrapper) -> Self {
        let branch: [ethers_core::types::H256; TREE_DEPTH] = wrapper.branch.map(|w| w.into());
        accumulator::NomadLightMerkle::new(branch, wrapper.count)
    }
}
