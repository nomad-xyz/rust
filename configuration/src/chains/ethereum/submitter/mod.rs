//! Ethereum tx submitter types

use crate::{agent::SignerConf, FromEnv};

mod gelato;
pub use gelato::*;

fn get_submitter_type(prefix: &str, default_prefix: Option<&str>) -> Option<String> {
    let mut submitter_type = std::env::var(&format!("{}_SUBMITTERTYPE", prefix)).ok();
    if let (None, Some(prefix)) = (&submitter_type, default_prefix) {
        submitter_type = std::env::var(&format!("{}_SUBMITTERTYPE", prefix)).ok();
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

impl FromEnv for TxSubmitterConf {
    fn from_env(prefix: &str, default_prefix: Option<&str>) -> Option<Self> {
        let submitter_type = get_submitter_type(prefix, default_prefix)?;

        return match submitter_type.as_ref() {
            "local" => {
                let default_prefix = default_prefix.map(|p| format!("{}_TXSIGNERS", p));
                let signer_conf = SignerConf::from_env(
                    &format!("{}_TXSIGNER", prefix),
                    default_prefix.as_deref(),
                )?;
                Some(Self::Local(signer_conf))
            }
            "gelato" => {
                let default_prefix = default_prefix.map(|p| format!("{}_GELATO", p));
                let gelato_conf =
                    GelatoConf::from_env(&format!("{}_GELATO", prefix), default_prefix.as_deref())?;
                Some(Self::Gelato(gelato_conf))
            }
            _ => panic!("Unknown tx submission type: {}", submitter_type),
        };
    }
}
