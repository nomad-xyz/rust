//! Ethereum tx submitter types

use crate::{agent::SignerConf, FromEnv};

mod gelato;
pub use gelato::*;

/// Local or relay-based transaction submission
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "submitterType", content = "submitter", rename_all = "camelCase")]
pub enum TxSubmitterConf {
    /// Signer configuration for local signer
    Local(SignerConf),
    /// Gelato configuration for Gelato relay
    Gelato(GelatoConf),
}

impl From<crate::TxSubmitterConf> for TxSubmitterConf {
    fn from(conf: crate::TxSubmitterConf) -> Self {
        let crate::TxSubmitterConf::Ethereum(conf) = conf;
        conf
    }
}

impl FromEnv for TxSubmitterConf {
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
