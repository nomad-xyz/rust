use serde::{Deserialize, Serialize};

/// Substrate-specific Nomad states
#[derive(Clone, Copy, Serialize, Deserialize)]
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
