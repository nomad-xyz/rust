//! Chain-specific configuration types

pub mod ethereum;

/// A connection to _some_ blockchain.
///
/// Specify the chain name (enum variant) in toml under the `chain` key
/// Specify the connection details as a toml object under the `connection` key.
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
#[serde(tag = "rpcStyle", content = "connection", rename_all = "camelCase")]
pub enum ChainConf {
    /// Ethereum configuration
    Ethereum(ethereum::Connection),
}

impl Default for ChainConf {
    fn default() -> Self {
        Self::Ethereum(Default::default())
    }
}
