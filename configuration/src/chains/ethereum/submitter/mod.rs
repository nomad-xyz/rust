//! Ethereum tx submitter types

use crate::agent::SignerConf;

mod gelato;
pub use gelato::*;

fn get_submitter_type(network: &str) -> Option<String> {
    let mut submitter_type = std::env::var(&format!("{}_SUBMITTERTYPE", network)).ok();
    if submitter_type.is_none() {
        submitter_type = std::env::var("DEFAULT_SUBMITTERTYPE").ok();
    }

    submitter_type
}

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

impl TxSubmitterConf {
    /// Build ethereum TxSubmitterConf from env. Looks for default submitter
    /// type if network-specific not defined.
    pub fn from_env(network: &str) -> Option<Self> {
        let submitter_type = get_submitter_type(network)?;

        return match submitter_type.as_ref() {
            "local" => {
                let signer_conf = SignerConf::from_env(Some("TXSIGNER"), Some(network))?;
                Some(Self::Local(signer_conf))
            }
            "gelato" => {
                let gelato_conf = GelatoConf::from_env(network)?;
                Some(Self::Gelato(gelato_conf))
            }
            _ => panic!("Unknown tx submission type: {}", submitter_type),
        };
    }
}
