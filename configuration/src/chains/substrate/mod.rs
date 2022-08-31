//! Substrate tx submitter types

use crate::agent::SignerConf;
use crate::TxSubmitterConf as BaseTxSubmitterConf;
use std::str::FromStr;

/// Substrate submitter type
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SubmitterType {
    /// Local sign/submit
    Local,
}

impl FromStr for SubmitterType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "local" => Ok(Self::Local),
            _ => panic!("Unknown SubmitterType"),
        }
    }
}

/// Local or relay-based transaction submission
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "submitterType", content = "submitter", rename_all = "camelCase")]
pub enum TxSubmitterConf {
    /// Signer configuration for local signer
    Local(SignerConf),
}

impl From<BaseTxSubmitterConf> for TxSubmitterConf {
    fn from(conf: BaseTxSubmitterConf) -> Self {
        match conf {
            BaseTxSubmitterConf::Substrate(conf) => conf,
            _ => panic!("Should never compile"),
        }
    }
}

impl TxSubmitterConf {
    /// Build Substrate TxSubmitterConf from env. Looks for default submitter
    /// type if network-specific not defined.
    pub fn from_env(network: &str) -> Option<Self> {
        let submitter_type = crate::utils::network_or_default_from_env(network, "SUBMITTER_TYPE")?;

        match SubmitterType::from_str(&submitter_type).unwrap() {
            SubmitterType::Local => {
                let signer_conf = SignerConf::from_env(Some("TXSIGNER"), Some(network))?;
                Some(Self::Local(signer_conf))
            }
        }
    }
}
