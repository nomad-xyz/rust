#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayerConfig {
    pub interval: u64,
}
