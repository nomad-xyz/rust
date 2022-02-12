use std::path::PathBuf;

use crate::common::NumberOrNumberString;

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

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    fmt: LogStyle,
    level: LogLevel,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexConfig {
    from: NumberOrNumberString,
    chunk: NumberOrNumberString,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseAgentConfig {
    enabled: bool,
    interval: NumberOrNumberString,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    rpc_style: RpcStyles,
    timelag: NumberOrNumberString,
    db: PathBuf,
    logging: LogConfig,
    index: IndexConfig,

    updater: BaseAgentConfig,
    relayer: BaseAgentConfig,
    processor: BaseAgentConfig,
    watcher: BaseAgentConfig,
    kathy: BaseAgentConfig,
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
