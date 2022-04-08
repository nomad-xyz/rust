//! Per-chain gas configurations

/// Gas settings for a given contract method
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GasSettings {
    /// Gas limit
    pub limit: u64,
    /// Gas price
    pub price: Option<u64>,
}

/// Gas settings specifically for a home update call
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeUpdateGasSettings {
    /// Per message additional gas cost
    pub per_message: u64,
    /// Base gas settings
    #[serde(flatten)]
    pub base: GasSettings,
}

/// Home gas settings
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeGasSettings {
    /// Update
    pub update: HomeUpdateGasSettings,
    /// Improper update
    pub improper_update: GasSettings,
    /// Double update
    pub double_update: GasSettings,
}

/// Replica gas settings
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaGasSettings {
    /// Update
    pub update: GasSettings,
    /// Prove
    pub prove: GasSettings,
    /// Process
    pub process: GasSettings,
    /// Double update
    pub double_update: GasSettings,
}

/// Connection manager gas settings
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionManagerGasSettings {
    /// Owner unenroll replica
    pub owner_unenroll_replica: GasSettings,
    /// Unenroll replica
    pub unenroll_replica: GasSettings,
}

/// Gas configuration for core contract methods
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoreGasConfig {
    /// Home gas settings
    pub home: HomeGasSettings,
    /// Replica Gas settings
    pub replica: ReplicaGasSettings,
    /// Connection manager gas settings
    pub connection_manager: ConnectionManagerGasSettings,
}
