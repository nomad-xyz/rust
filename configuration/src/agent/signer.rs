use std::str::FromStr;

use crate::FromEnv;
use nomad_types::HexString;

/// Ethereum signer types
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum SignerConf {
    /// A local hex key, hex string of private key, with or without 0x prefix
    HexKey(HexString<64>),
    /// An AWS signer. Note that AWS credentials must be inserted into the env
    /// separately.
    Aws {
        /// An AWS identifier for the key. This may be
        /// 1. Its UUID
        /// 2. Its ARN
        /// 3. A key alias
        /// 4. A key alias's ARN
        id: String,
    },
    /// Assume node will sign on RPC calls
    Node,
}

impl Default for SignerConf {
    fn default() -> Self {
        Self::Node
    }
}

impl FromEnv for SignerConf {
    fn from_env(prefix: &str, default_prefix: Option<&str>) -> Option<Self> {
        // ordering this first preferentially uses AWS if both are specified
        if let Ok(id) = std::env::var(&format!("{}_ID", prefix)) {
            return Some(SignerConf::Aws { id });
        }

        if let Ok(signer_key) = std::env::var(&format!("{}_KEY", prefix)) {
            return Some(SignerConf::HexKey(HexString::from_str(&signer_key).ok()?));
        }

        if let Some(prefix) = default_prefix {
            return SignerConf::from_env(prefix, None);
        }

        None
    }
}

#[cfg(test)]
mod test {
    use serde_json::{json, Value};

    use super::SignerConf;

    #[test]
    fn it_deserializes_hexkey_signer_confs() {
        let k = "0x3232323232323232323232323232323232323232323232323232323232323232";
        let value = json! { "0x3232323232323232323232323232323232323232323232323232323232323232" };

        let signer_conf: SignerConf = serde_json::from_value(value).unwrap();
        assert_eq!(signer_conf, SignerConf::HexKey(k.parse().unwrap()));

        let value = Value::Null;
        let signer_conf: SignerConf = serde_json::from_value(value).unwrap();
        assert_eq!(signer_conf, SignerConf::Node);
    }
    #[test]
    fn it_deserializes_aws_signer_confs() {
        let value = json!({
            "id": "",
        });
        let signer_conf: SignerConf = serde_json::from_value(value).unwrap();
        assert_eq!(signer_conf, SignerConf::Aws { id: "".to_owned() });
    }
}
