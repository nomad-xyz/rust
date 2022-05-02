use crate::{agent::SignerConf, FromEnv};

/// Configuration for tx submission through Gelato relay
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoConf {
    /// Sponsor signer configuration
    pub signer: SignerConf,
    /// Address of fee token
    pub fee_token: String,
}

impl FromEnv for GelatoConf {
    fn from_env(prefix: &str) -> Option<Self> {
        let signer = SignerConf::from_env(&format!("{}_SIGNER", prefix))?;
        let fee_token = std::env::var(&format!("{}_FEETOKEN", prefix)).ok()?;

        Some(Self { signer, fee_token })
    }
}
