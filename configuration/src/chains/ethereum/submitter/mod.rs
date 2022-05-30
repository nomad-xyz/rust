//! Ethereum tx submitter types

use crate::agent::SignerConf;

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

impl TxSubmitterConf {
    /// Build ethereum TxSubmitterConf from env. Looks for default submitter
    /// type if network-specific not defined.
    pub fn from_env(network: &str) -> Option<Self> {
        let submitter_type = crate::utils::network_or_default_from_env(network, "SUBMITTER_TYPE")?;

        return match submitter_type.to_lowercase().as_ref() {
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
