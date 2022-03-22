//! Agent configuration types

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
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
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
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    /// fmt specifier
    pub fmt: LogStyle,
    /// level specifier
    pub level: LogLevel,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            fmt: LogStyle::Pretty,
            level: LogLevel::Trace,
        }
    }
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
