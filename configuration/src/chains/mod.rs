//! Chain-specific configuration types

pub mod ethereum;

use std::str::FromStr;

use serde_json::json;

/// Rpc style of chain
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RpcStyle {
    /// Ethereum
    Ethereum,
}

impl FromStr for RpcStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "ethereum" => Ok(Self::Ethereum),
            _ => panic!("Unknown RpcStyle"),
        }
    }
}

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

impl ChainConf {
    /// Build ChainConf from env vars. Will use default RPCSTYLE if
    /// network-specific not provided.
    pub fn from_env(network: &str) -> Option<Self> {
        let mut rpc_style = std::env::var(&format!("{}_RPCSTYLE", network)).ok();

        if rpc_style.is_none() {
            rpc_style = std::env::var("DEFAULT_RPCSTYLE").ok();
        }

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

impl TxSubmitterConf {
    /// Build TxSubmitterConf from env. Looks for default RPC style if
    /// network-specific not defined.
    pub fn from_env(network: &str) -> Option<Self> {
        let rpc_style = crate::utils::network_or_default_from_env(network, "RPCSTYLE")?;

        match RpcStyle::from_str(&rpc_style).unwrap() {
            RpcStyle::Ethereum => Some(Self::Ethereum(ethereum::TxSubmitterConf::from_env(
                network,
            )?)),
        }
    }
}
