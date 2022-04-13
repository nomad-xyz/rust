//! Per-chain gas configurations
/// Gas settings specifically for a home update call
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeUpdateGasLimit {
    /// Per message additional gas cost
    pub per_message: u64,
    /// Base gas settings
    pub base: u64,
}

/// Home gas settings
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

/// Replica gas settings
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

/// Connection manager gas settings
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionManagerGasLimits {
    /// Owner unenroll replica
    pub owner_unenroll_replica: u64,
    /// Unenroll replica
    pub unenroll_replica: u64,
}

/// Gas configuration for core contract methods
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoreGasConfig {
    /// Home gas settings
    pub home: HomeGasLimits,
    /// Replica Gas settings
    pub replica: ReplicaGasLimits,
    /// Connection manager gas settings
    pub connection_manager: ConnectionManagerGasLimits,
}
