//! Ethereum/EVM configuration types

use std::str::FromStr;

mod submitter;
pub use submitter::*;

/// Ethereum connection configuration
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

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::Connection;

    #[test]
    fn it_desers_ethereum_rpc_configs() {
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
