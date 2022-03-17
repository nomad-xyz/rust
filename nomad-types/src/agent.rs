//! Agent configuration types

use std::path::PathBuf;

use crate::core::deser_nomad_number;

/// Rpc Styles
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RpcStyles {
    /// Ethereum
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

/// Logger configuration
#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    /// fmt specifier
    pub fmt: LogStyle,
    /// level specifier
    pub level: LogLevel,
}

/// Individual agent configuration
#[derive(Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseAgentConfig {
    /// true if the agent should be run
    pub enabled: bool,
    /// the polling interval
    #[serde(deserialize_with = "deser_nomad_number")]
    pub interval: u64,
}

/// Full agent configuration
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    /// RPC specifier
    pub rpc_style: RpcStyles,
    /// Path to the DB
    pub db: PathBuf,
    /// Logging configuration
    pub logging: LogConfig,
    /// Updater configuration
    pub updater: BaseAgentConfig,
    /// Relayer configuration
    pub relayer: BaseAgentConfig,
    /// Processor configuration
    pub processor: BaseAgentConfig,
    /// Watcher configuration
    pub watcher: BaseAgentConfig,
    /// Kathy configuration
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
