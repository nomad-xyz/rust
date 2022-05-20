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
    fn from_env(prefix: &str, default_prefix: Option<&str>) -> Option<Self> {
        if let Some(sponsor) = SignerConf::from_env(&format!("{}_SPONSOR", prefix), None) {
            if let Some(fee_token) = std::env::var(&format!("{}_FEETOKEN", prefix)).ok() {
                return Some(Self { sponsor, fee_token });
            }
        }

        if let Some(prefix) = default_prefix {
            return GelatoConf::from_env(prefix, None);
        }

        None
    }
}
