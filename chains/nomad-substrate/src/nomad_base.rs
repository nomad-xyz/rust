use crate::NomadState;
use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct H256Wrapper([H256; 1]);
impl std::ops::Deref for H256Wrapper {
    type Target = H256;
    fn deref(&self) -> &Self::Target {
        &self.0[0]
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct H160Wrapper([H160; 1]);
impl std::ops::Deref for H160Wrapper {
    type Target = H160;
    fn deref(&self) -> &Self::Target {
        &self.0[0]
    }
}

/// NomadBase struct stored in Substrate home
#[derive(Clone, Copy, Serialize, Deserialize)]
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
