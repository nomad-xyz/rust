//! Per-chain gas configurations

/// Gas configuration for core and bridge contract methods
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NomadGasConfig {
    /// Core gas limits
    pub core: CoreGasConfig,
    /// Bridge gas limits
    pub bridge: BridgeGasConfig,
}

/// Gas configuration for core contract methods
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
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
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeUpdateGasLimit {
    /// Per message additional gas cost
    pub per_message: u64,
    /// Base gas limits
    pub base: u64,
}

/// Home gas limits
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
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
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
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
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionManagerGasLimits {
    /// Owner unenroll replica
    pub owner_unenroll_replica: u64,
    /// Unenroll replica
    pub unenroll_replica: u64,
}

/// Gas configuration for bridge contract methods
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BridgeGasConfig {
    /// BridgeRouter gas limits
    pub bridge_router: BridgeRouterGasLimits,
    /// EthHelper gas limits
    pub eth_helper: EthHelperGasLimits,
}

/// Gas limits for BridgeRouter
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BridgeRouterGasLimits {
    /// Send
    pub send: u64,
}

/// Gas limits for EthHelper
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EthHelperGasLimits {
    /// Send
    pub send: u64,
    /// Send to EVM like
    pub send_to_evm_like: u64,
}
