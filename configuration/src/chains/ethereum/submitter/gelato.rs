use crate::{agent::SignerConf, FromEnv};

/// Configuration for tx submission through Gelato relay
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoConf {
    /// Sponsor signer configuration
    pub sponsor: SignerConf,
    /// Address of fee token
    pub fee_token: String,
}

impl FromEnv for GelatoConf {
    fn from_env(prefix: &str) -> Option<Self> {
        let sponsor = SignerConf::from_env(&format!("{}_SPONSOR", prefix))?;
        let fee_token = std::env::var(&format!("{}_FEETOKEN", prefix)).ok()?;

        Some(Self { sponsor, fee_token })
    }
}
