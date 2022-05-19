use std::str::FromStr;

use crate::FromEnv;
use nomad_types::HexString;

/// Ethereum signer configurations.
///
/// This config item specifies valid Ethereum signers.
/// When deserializing, it attempts the following:
/// 1. Deserialize the value as a 32-byte hex string
///    A hex string is treated as a private key for a local signer. This is
///    generally discouraged.
/// 2. Deserialize the value as an object containing a single key `id`,
///    whose value is a string. This is treated as an identifier for an AWS
///    key. If this configuration is used, the AWS region and credentials must
///    be supplied when running the program. Typically these are inserted by
///    env var, aws config, or instance roles.
/// 3. Anything else is treated as an instruction to request the RPC node sign
///    transactions and messages via the `eth_sign` family of RPC requests. If
///    this mode is used, the RPC mode must be unlocked, and have a key.
///
/// # Examples
///
/// ```ignore
/// // JSON examples
/// // Hex Key
/// "0x1234123412341234123412341234123412341234123412341234123412341234"
/// // Aws
/// { "id": "5485edfa-d7c2-11ec-9d64-0242ac120002" }
/// // Node signer
/// null
/// "asdjf"
/// 38
/// ```
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
