use nomad_types::HexString;

/// Ethereum signer types
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum SignerConf {
    /// A local hex key
    HexKey(
        /// Hex string of private key, without 0x prefix
        HexString<64>,
    ),
    /// An AWS signer. Note that AWS credentials must be inserted into the env
    /// separately.
    Aws {
        /// The UUID identifying the AWS KMS Key
        id: String, // change to no _ so we can set by env
        /// The AWS region
        region: String,
    },
    /// Assume node will sign on RPC calls
    Node,
}

impl Default for SignerConf {
    fn default() -> Self {
        Self::Node
    }
}

#[cfg(test)]
mod test {
    use serde_json::{json, Value};

    use super::SignerConf;

    #[test]
    fn it_deserializes_signer_confs() {
        let k = "0x3232323232323232323232323232323232323232323232323232323232323232";
        let value = json! { "0x3232323232323232323232323232323232323232323232323232323232323232" };

        dbg!(&value);

        let signer_conf: SignerConf = serde_json::from_value(value).unwrap();
        assert_eq!(signer_conf, SignerConf::HexKey(k.parse().unwrap()));

        let value = Value::Null;
        let signer_conf: SignerConf = serde_json::from_value(value).unwrap();
        assert_eq!(signer_conf, SignerConf::Node);

        let value = json!({
            "id": "",
            "region": ""
        });
        let signer_conf: SignerConf = serde_json::from_value(value).unwrap();
        assert_eq!(
            signer_conf,
            SignerConf::Aws {
                id: "".to_owned(),
                region: "".to_owned()
            }
        );
    }
}
