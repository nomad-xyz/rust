//! Chain-specific configuration types

pub mod ethereum;

use crate::FromEnv;
use serde_json::json;

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

impl FromEnv for ChainConf {
    fn from_env(network: &str) -> Option<Self> {
        let rpc_style = std::env::var(&format!("{}_RPCSTYLE", network)).ok()?;
        let rpc_url = std::env::var(&format!("{}_CONNECTION_URL", network)).ok()?;

        let json = json!({
            "rpcStyle": rpc_style,
            "connection": rpc_url,
        });

        Some(
            serde_json::from_value(json)
                .unwrap_or_else(|_| panic!("malformed json for {} rpc", network)),
        )
    }
}

/// Transaction submssion configuration for some chain.
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
#[serde(tag = "rpcStyle", rename_all = "camelCase")]
pub enum TxSubmitterConf {
    /// Ethereum configuration
    Ethereum(ethereum::TxSubmitterConf),
}

impl FromEnv for TxSubmitterConf {
    fn from_env(network: &str) -> Option<Self> {
        let rpc_style = std::env::var(&format!("{}_RPCSTYLE", network)).ok()?;

        match rpc_style.as_ref() {
            "ethereum" => Some(Self::Ethereum(ethereum::TxSubmitterConf::from_env(
                network,
            )?)),
            _ => panic!("Unknown transaction submission rpc style: {}", rpc_style),
        }
    }
}
