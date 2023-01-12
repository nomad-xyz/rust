//! Chain-specific configuration types

pub mod ethereum;

pub mod substrate;

use std::str::FromStr;

use serde_json::json;

/// Rpc style of chain
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RpcStyle {
    /// Ethereum
    Ethereum,
    /// Substrate
    Substrate,
}

impl FromStr for RpcStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "ethereum" => Ok(Self::Ethereum),
            "substrate" => Ok(Self::Substrate),
            _ => panic!("Unknown RpcStyle"),
        }
    }
}

/// Chain connection configuration
#[derive(Debug, Clone, PartialEq)]
pub enum Connection {
    /// HTTP connection details
    Http(
        /// Fully qualified URI to connect to
        String,
    ),
    /// Websocket connection details
    Ws(
        /// Fully qualified URI to connect to
        String,
    ),
}

impl Connection {
    fn from_string(s: String) -> eyre::Result<Self> {
        if s.starts_with("http://") || s.starts_with("https://") {
            Ok(Self::Http(s))
        } else if s.starts_with("wss://") || s.starts_with("ws://") {
            Ok(Self::Ws(s))
        } else {
            eyre::bail!("Expected http or websocket URI")
        }
    }
}

impl FromStr for Connection {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s.to_owned())
    }
}

impl Default for Connection {
    fn default() -> Self {
        Self::Http(Default::default())
    }
}

impl<'de> serde::Deserialize<'de> for Connection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_string(s).map_err(serde::de::Error::custom)
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
    Ethereum(Connection),
    /// Substrate configuration
    Substrate(Connection),
}

impl Default for ChainConf {
    fn default() -> Self {
        Self::Ethereum(Default::default())
    }
}


impl ChainConf {
    /// Build ChainConf from env vars. Will use default RPCSTYLE if
    /// network-specific not provided.
    #[tracing::instrument]
    pub fn from_env(network: &str) -> Option<Self> {
        let style_key = &format!("{}_RPCSTYLE", network);
        let default_style_key = "DEFAULT_RPCSTYLE";
        let rpc_style = std::env::var(&style_key)
            .or_else(|_| {
                tracing::debug!("falling back to env default rpc style");
                std::env::var(default_style_key)
            })
            .unwrap_or_else(|_| {
                tracing::debug!("falling back to ethereum");
                "etherum".to_owned()
            });

        let rpc_url: Connection = std::env::var(&format!("{}_CONNECTION_URL", network))
            .map(|url| {
                tracing::debug!(url, "connection url env var read");
                url
            })
            .ok()?
            .parse()
            .map_err(|e: eyre::Report| {
                tracing::error!(err = e.to_string(), "unable to parse connection url")
            })
            .ok()?;

        Some(match rpc_style.as_str() {
            "substrate" => ChainConf::Substrate(rpc_url),
            "ethereum" => ChainConf::Ethereum(rpc_url),
            _ => panic!("Invalid rpc style {}", rpc_style),
        })
    }
}


/// Transaction submssion configuration for some chain.
#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
#[serde(tag = "rpcStyle", rename_all = "camelCase")]
pub enum TxSubmitterConf {
    /// Ethereum configuration
    Ethereum(ethereum::TxSubmitterConf),
    /// Substrate configuration
    Substrate(substrate::TxSubmitterConf),
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
            RpcStyle::Substrate => Some(Self::Substrate(substrate::TxSubmitterConf::from_env(
                network,
            )?)),
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::Connection;

    #[test]
    fn it_desers_rpc_configs() {
        let value = json! {
            "https://google.com"
        };
        let connection: Connection = serde_json::from_value(value).unwrap();
        assert_eq!(
            connection,
            Connection::Http("https://google.com".to_owned())
        );
        let value = json! {
            "http://google.com"
        };
        let connection: Connection = serde_json::from_value(value).unwrap();
        assert_eq!(connection, Connection::Http("http://google.com".to_owned()));
        let value = json! {
            "wss://google.com"
        };
        let connection: Connection = serde_json::from_value(value).unwrap();
        assert_eq!(connection, Connection::Ws("wss://google.com".to_owned()));
        let value = json! {
            "ws://google.com"
        };
        let connection: Connection = serde_json::from_value(value).unwrap();
        assert_eq!(connection, Connection::Ws("ws://google.com".to_owned()));
    }
}
