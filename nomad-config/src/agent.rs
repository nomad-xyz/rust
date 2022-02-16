use std::path::PathBuf;

use crate::common::deser_nomad_number;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RpcStyles {
    Ethereum,
}

impl Default for RpcStyles {
    fn default() -> Self {
        RpcStyles::Ethereum
    }
}

/// Basic tracing configuration
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LogStyle {
    /// Pretty print
    Pretty,
    /// JSON
    Json,
    /// Compact
    Compact,
    /// Default style
    #[serde(other)]
    Full,
}

impl Default for LogStyle {
    fn default() -> Self {
        LogStyle::Full
    }
}

/// Logging level
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    /// Off
    Off,
    /// Error
    Error,
    /// Warn
    Warn,
    /// Debug
    Debug,
    /// Trace
    Trace,
    /// Info
    #[serde(other)]
    Info,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    pub fmt: LogStyle,
    pub level: LogLevel,
}

#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexConfig {
    #[serde(deserialize_with = "deser_nomad_number")]
    pub from: u64,
    #[serde(deserialize_with = "deser_nomad_number")]
    pub chunk: u64,
}

#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseAgentConfig {
    pub enabled: bool,
    #[serde(deserialize_with = "deser_nomad_number")]
    pub interval: u64,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub rpc_style: RpcStyles,
    #[serde(deserialize_with = "deser_nomad_number")]
    pub timelag: u64,
    pub db: PathBuf,
    pub logging: LogConfig,
    pub index: IndexConfig,

    pub updater: BaseAgentConfig,
    pub relayer: BaseAgentConfig,
    pub processor: BaseAgentConfig,
    pub watcher: BaseAgentConfig,
    pub kathy: BaseAgentConfig,
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::RpcStyles;

    #[test]
    fn it_deserializes_rpc_styles() {
        let serialized = serde_json::to_value(&RpcStyles::Ethereum).unwrap();

        let val = json! { "ethereum" };
        assert_eq!(val, serialized);
    }
}
