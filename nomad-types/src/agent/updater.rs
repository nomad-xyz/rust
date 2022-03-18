#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterSettingsBlock {
    pub interval: u64,
}
