//! Ethereum/EVM configuration types

use crate::{agent::SignerConf, FromEnv};

mod gelato;
pub use gelato::*;

/// Ethereum connection configuration
#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
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

impl Default for Connection {
    fn default() -> Self {
        Self::Http {
            url: Default::default(),
        }
    }
}

/// Local or relay-based transaction submission
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "submitterType", content = "submitter", rename_all = "camelCase")]
pub enum TransactionSubmitterConf {
    /// Signer configuration for local signer
    Local(SignerConf),
    /// Gelato configuration for Gelato relay
    Gelato(GelatoConf),
}

impl From<super::TransactionSubmitterConf> for TransactionSubmitterConf {
    fn from(conf: super::TransactionSubmitterConf) -> Self {
        let super::TransactionSubmitterConf::Ethereum(conf) = conf;
        conf
    }
}

impl FromEnv for TransactionSubmitterConf {
    fn from_env(prefix: &str) -> Option<Self> {
        let submitter_type = std::env::var(&format!("{}_SUBMITTERTYPE", prefix)).ok()?;

        match submitter_type.as_ref() {
            "local" => {
                let signer_conf = SignerConf::from_env(&format!("{}_SUBMITTER", prefix))?;
                Some(Self::Local(signer_conf))
            }
            "gelato" => {
                let gelato_conf = GelatoConf::from_env(&format!("{}_SUBMITTER", prefix))?;
                Some(Self::Gelato(gelato_conf))
            }
            _ => panic!("Unknown tx submission type: {}", submitter_type),
        }
    }
}
