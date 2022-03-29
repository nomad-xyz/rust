//! Ethereum/EVM configuration types

use std::str::FromStr;

/// Ethereum connection configuration
#[derive(Debug, Clone, PartialEq)]
pub enum Connection {
    /// HTTP connection details
    Http {
        /// Fully qualified string to connect to
        url: String,
    },
    /// Websocket connection details
    Ws {
        /// Fully qualified string to connect to
        url: String,
    },
}

impl Connection {
    fn from_string(s: String) -> eyre::Result<Self> {
        if s.starts_with("http://") || s.starts_with("https://") {
            Ok(Self::Http { url: s })
        } else if s.starts_with("wss://") || s.starts_with("ws://") {
            Ok(Self::Ws { url: s })
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

impl<'de> serde::Deserialize<'de> for Connection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_string(s).map_err(serde::de::Error::custom)
    }
}

impl Default for Connection {
    fn default() -> Self {
        Self::Http {
            url: Default::default(),
        }
    }
}
