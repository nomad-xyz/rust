use crate::NomadIdentifier;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherConfig {
    pub interval: u64,
    pub managers: HashMap<String, NomadIdentifier>,
}
