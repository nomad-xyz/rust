use std::fmt;

use ethers::prelude::{Address, H256};
use serde::{de, Deserializer};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, serde::Serialize, Default, Hash)]
pub struct NomadIdentifier(H256);

impl<'de> serde::Deserialize<'de> for NomadIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NomadIdentifierVisitor)
    }
}

impl From<H256> for NomadIdentifier {
    fn from(h: H256) -> Self {
        Self(h)
    }
}

impl From<Address> for NomadIdentifier {
    fn from(h: Address) -> Self {
        Self(h.into())
    }
}

struct NomadIdentifierVisitor;

impl<'de> de::Visitor<'de> for NomadIdentifierVisitor {
    type Value = NomadIdentifier;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a 20- or 32-byte 0x-prepended hexadecimal string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(h) = v.parse::<H256>() {
            return Ok(h.into());
        }
        if let Ok(a) = v.parse::<Address>() {
            return Ok(a.into());
        }

        Err(E::custom("Unable to parse H256 or Address from string"))
    }
}

pub fn deser_nomad_number<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct NumberOrNumberStringVisitor;

    impl<'de> de::Visitor<'de> for NumberOrNumberStringVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter
                .write_str("an integer, a decimal string, or a 0x-prepended hexadecimal string")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if let Ok(res) = v.parse() {
                return Ok(res);
            }

            if v.starts_with("0x") {
                if v.len() == 2 {
                    return Ok(0);
                }
                if let Ok(res) = u64::from_str_radix(&v[2..], 16) {
                    return Ok(res);
                }
            }

            Err(E::invalid_value(de::Unexpected::Str(v), &self))
        }
    }

    deserializer.deserialize_any(NumberOrNumberStringVisitor)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum NameOrDomain {
    Name(String),
    Domain(u32),
}

impl From<String> for NameOrDomain {
    fn from(s: String) -> Self {
        Self::Name(s)
    }
}

impl From<u32> for NameOrDomain {
    fn from(t: u32) -> Self {
        Self::Domain(t.into())
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn it_sers_and_desers_identifiers() {
        let addr = json! {"0x0000000000000000000000000000000000000000"};
        let h256 = json! {"0x0000000000000000000000000000000000000000000000000000000000000000"};

        let expected = NomadIdentifier::default();
        assert_eq!(h256, serde_json::to_value(&expected).unwrap());

        let a: NomadIdentifier = serde_json::from_value(addr).unwrap();
        let b = serde_json::from_value(h256).unwrap();
        assert_eq!(a, b);
        assert_eq!(a, expected);
    }

    #[test]
    fn it_sers_and_desers_numbers() {
        let five: u64 = 5;
        let serialized = serde_json::to_value(&five).unwrap();

        let val = json! { 5 };
        assert_eq!(serialized, val);
        let n = deser_nomad_number(val).unwrap();
        assert_eq!(n, five);

        let val = json! { "5" };
        let n = deser_nomad_number(val).unwrap();
        assert_eq!(n, five);

        let val = json! { "0x5" };
        let n = deser_nomad_number(val).unwrap();
        assert_eq!(n, five);
    }
}
