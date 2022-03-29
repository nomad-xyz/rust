use nomad_types::HexString;
use serde::{
    de::{self},
    Deserialize,
};

/// Ethereum signer types
#[derive(Debug, Clone, PartialEq)]
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
    /// Assume node will sign on RPC calls
    Node,
}

impl<'de> serde::Deserialize<'de> for SignerConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum AwsField {
            Id,
            Region,
        }

        struct SignerConfVisitor;

        impl<'de> de::Visitor<'de> for SignerConfVisitor {
            type Value = SignerConf;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "A 32-byte 0x-prefixed hex-key, OR an aws key id and regior OR nothing",
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let key = HexString::<64>::from_string(v).map_err(de::Error::custom)?;
                Ok(SignerConf::HexKey { key })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut id: Option<String> = None;
                let mut region: Option<String> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        AwsField::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        AwsField::Region => {
                            if region.is_some() {
                                return Err(de::Error::duplicate_field("region"));
                            }
                            region = Some(map.next_value()?);
                        }
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let region = region.ok_or_else(|| de::Error::missing_field("region"))?;
                Ok(SignerConf::Aws { id, region })
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SignerConf::Node)
            }
        }

        const VARIANTS: &'static [&'static str] = &["hexkey", "aws", "node"];
        deserializer.deserialize_enum("SignerConf", VARIANTS, SignerConfVisitor)
    }
}

impl Default for SignerConf {
    fn default() -> Self {
        Self::Node
    }
}
