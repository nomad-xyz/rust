use crate::agent::SignerConf;

/// Configuration for tx submission through Gelato relay
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoConf {
    /// Sponsor signer configuration
    pub sponsor: SignerConf,
    /// Address of fee token
    pub fee_token: String,
}

impl GelatoConf {
    /// Build GelatoConf from env. Looks for default configuration if
    /// network-specific not defined.
    pub fn from_env(network: &str) -> Option<Self> {
        Self::from_full_prefix(network).or_else(|| Self::from_full_prefix("DEFAULT"))
    }

    fn from_full_prefix(network: &str) -> Option<Self> {
        if let Some(sponsor) = SignerConf::from_env(Some("GELATO_SPONSOR"), Some(network)) {
            if let Ok(fee_token) = std::env::var(&format!("{}_GELATO_FEETOKEN", network)) {
                return Some(Self { sponsor, fee_token });
            }
        }

        None
    }
}
