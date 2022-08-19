use crate::NomadState;
use serde::{Deserialize, Serialize};
use subxt::ext::sp_core::{H160, H256};

/// NomadBase struct stored in Substrate home
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct NomadBase {
    /// State
    pub state: NomadState,
    /// Local domain
    pub local_domain: u32,
    /// Committed root
    pub committed_root: H256,
    /// Updater
    pub updater: H160,
}
