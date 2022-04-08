//! Per-chain gas configurations

/// Gas settings for a given contract method
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GasSettings {
    limit: u64,
    price: Option<u64>,
}

/// Gas settings specifically for a home update call
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeUpdateGasSettings {
    per_message: u64,
    #[serde(flatten)]
    base: GasSettings,
}

/// Home gas settings
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeGasSettings {
    update: HomeUpdateGasSettings,
    improper_update: GasSettings,
    double_update: GasSettings,
}

/// Replica gas settings
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaGasSettings {
    update: GasSettings,
    prove: GasSettings,
    process: GasSettings,
    double_update: GasSettings,
}

/// Connection manager gas settings
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionManagerGasSettings {
    owner_unenroll_replica: GasSettings,
    unenroll_replica: GasSettings,
}

/// Gas configuration for core contract methods
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoreGasConfig {
    home: HomeGasSettings,
    replica: ReplicaGasSettings,
    connection_manager: ConnectionManagerGasSettings,
}
