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
#[serde(tag = "submitterType", rename_all = "camelCase")]
pub enum TransactionSubmitter {
    /// Signer configuration for local signer
    Local(SignerConf),
    /// Gelato configuration for Gelato relay
    Gelato(GelatoConf),
}

impl FromEnv for TransactionSubmitter {
    fn from_env(network: &str) -> Option<Self> {
        let submission_type =
            std::env::var(&format!("TRANSACTIONSUBMITTER_{}_TYPE", network)).ok()?;

        match submission_type.as_ref() {
            "local" => {
                let signer_conf = SignerConf::from_env(&format!("TRANSACTIONSIGNERS_{}", network))?;
                Some(Self::Local(signer_conf))
            }
            "gelato" => {
                let gelato_conf = GelatoConf::from_env(network)?;
                Some(Self::Gelato(gelato_conf))
            }
            _ => panic!("Unknown tx submission type: {}", submission_type),
        }
    }
}
