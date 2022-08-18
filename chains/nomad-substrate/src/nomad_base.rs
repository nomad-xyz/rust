use crate::NomadState;
use serde::{Deserialize, Serialize};
use subxt::ext::sp_core::{H160, H256};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct NomadBase {
    pub state: NomadState,
    pub local_domain: u32,
    pub committed_root: H256,
    pub updater: H160,
}
