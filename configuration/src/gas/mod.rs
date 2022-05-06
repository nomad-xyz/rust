//! Per-chain gas configurations

use serde::{Deserialize, Serialize};

mod defaults;

/// Gas config types
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "config", rename_all = "camelCase")]
pub enum NomadGasConfigs {
    /// Custom fully specified
    Custom(NomadGasConfig),
    /// Evm default
    EvmDefault(defaults::EvmDefaultWrapper),
}

/// Gas configuration for core and bridge contract methods
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NomadGasConfig {
    /// Core gas limits
    pub core: CoreGasConfig,
    /// Bridge gas limits
    pub bridge: BridgeGasConfig,
}

impl NomadGasConfig {
    /// Return standard EVM gas values
    pub fn evm_default() -> Self {
        Self {
            core: CoreGasConfig {
                home: HomeGasLimits {
                    update: HomeUpdateGasLimit {
                        per_message: 10_000,
                        base: 100_000,
                    },
                    improper_update: HomeUpdateGasLimit {
                        per_message: 10_000,
                        base: 100_000,
                    },
                    double_update: 200_000,
                },
                replica: ReplicaGasLimits {
                    update: 140_000,
                    prove: 200_000,
                    process: 1_700_000,
                    prove_and_process: 1_900_000,
                    double_update: 200_000,
                },
                connection_manager: ConnectionManagerGasLimits {
                    owner_unenroll_replica: 120_000,
                    unenroll_replica: 120_000,
                },
            },
            bridge: BridgeGasConfig {
                bridge_router: BridgeRouterGasLimits { send: 500_000 },
                eth_helper: EthHelperGasLimits {
                    send: 800_000,
                    send_to_evm_like: 800_000,
                },
            },
        }
    }
}

/// Gas configuration for core contract methods
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoreGasConfig {
    /// Home gas limits
    pub home: HomeGasLimits,
    /// Replica gas limits
    pub replica: ReplicaGasLimits,
    /// Connection manager gas limits
    pub connection_manager: ConnectionManagerGasLimits,
}

/// Gas limits specifically for a home update call
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeUpdateGasLimit {
    /// Per message additional gas cost
    pub per_message: u64,
    /// Base gas limits
    pub base: u64,
}

/// Home gas limits
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeGasLimits {
    /// Update
    pub update: HomeUpdateGasLimit,
    /// Improper update
    pub improper_update: HomeUpdateGasLimit,
    /// Double update
    pub double_update: u64,
}

/// Replica gas limits
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaGasLimits {
    /// Update
    pub update: u64,
    /// Prove
    pub prove: u64,
    /// Process
    pub process: u64,
    /// Prove and process
    pub prove_and_process: u64,
    /// Double update
    pub double_update: u64,
}

/// Connection manager gas limits
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionManagerGasLimits {
    /// Owner unenroll replica
    pub owner_unenroll_replica: u64,
    /// Unenroll replica
    pub unenroll_replica: u64,
}

/// Gas configuration for bridge contract methods
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BridgeGasConfig {
    /// BridgeRouter gas limits
    pub bridge_router: BridgeRouterGasLimits,
    /// EthHelper gas limits
    pub eth_helper: EthHelperGasLimits,
}

/// Gas limits for BridgeRouter
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BridgeRouterGasLimits {
    /// Send
    pub send: u64,
}

/// Gas limits for EthHelper
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EthHelperGasLimits {
    /// Send
    pub send: u64,
    /// Send to EVM like
    pub send_to_evm_like: u64,
}
