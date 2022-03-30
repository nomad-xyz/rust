use crate::FromEnv;
use nomad_types::HexString;
use serde_json::json;

/// Ethereum signer types
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SignerConf {
    /// A local hex key
    HexKey {
        /// Hex string of private key, without 0x prefix
        key: HexString<64>,
    },
    /// An AWS signer. Note that AWS credentials must be inserted into the env
    /// separately.
    Aws {
        /// The UUID identifying the AWS KMS Key
        id: String, // change to no _ so we can set by env
        /// The AWS region
        region: String,
    },
    #[serde(other)]
    /// Assume node will sign on RPC calls
    Node,
}

impl Default for SignerConf {
    fn default() -> Self {
        Self::Node
    }
}

impl FromEnv for SignerConf {
    fn from_env(prefix: &str) -> Option<Self> {
        let signer_type = std::env::var(&format!("{}_TYPE", prefix)).ok()?;
        let signer_json = match signer_type.as_ref() {
            "hexKey" => {
                let signer_key = std::env::var(&format!("{}_KEY", prefix)).ok()?;
                json!({
                    "type": signer_type,
                    "key": signer_key,
                })
            }
            "aws" => {
                let id = std::env::var(&format!("{}_ID", prefix)).ok()?;
                let region = std::env::var(&format!("{}_REGION", prefix)).ok()?;
                json!({
                    "type": signer_type,
                    "id": id,
                    "region": region,
                })
            }
            _ => panic!("Unknown signer type: {}", signer_type),
        };

        Some(
            serde_json::from_value(signer_json)
                .unwrap_or_else(|_| panic!("malformed json for {} signer", prefix)),
        )
    }
}
