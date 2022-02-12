use std::fmt;

use ethers::prelude::{Address, H256};
use serde::de;

/// This type allows config files to express full-length (32-byte) identifiers,
/// or EVM-length (20-byte) identifiers
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum NomadIdentifier {
    Address(Address),
    H256(H256),
}

impl Default for NomadIdentifier {
    fn default() -> Self {
        NomadIdentifier::H256(Default::default())
    }
}

impl serde::Serialize for NomadIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_h256().serialize(serializer)
    }
}

impl NomadIdentifier {
    /// Converts to a H256 regardless of internal size
    pub fn as_h256(&self) -> H256 {
        match self {
            NomadIdentifier::Address(addr) => (*addr).into(),
            NomadIdentifier::H256(h) => *h,
        }
    }

    /// Attempt to convert to an evm address. Returns `None` if doing so would
    /// drop non-0 bytes from the front of the identifier
    pub fn as_evm_address(&self) -> Option<Address> {
        let zero_prefix = &[0u8; 12];
        match self {
            NomadIdentifier::Address(a) => Some(*a),
            NomadIdentifier::H256(h) => {
                if h.as_bytes().starts_with(zero_prefix) {
                    Some((*h).into())
                } else {
                    None
                }
            }
        }
    }
}

impl std::hash::Hash for NomadIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_h256().hash(state)
    }
}

impl PartialEq for NomadIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.as_h256() == other.as_h256()
    }
}

impl Eq for NomadIdentifier {}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct NumberOrNumberString(u64);

impl serde::Serialize for NumberOrNumberString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for NumberOrNumberString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let number = deserializer.deserialize_any(NumberOrNumberStringVisitor)?;
        Ok(NumberOrNumberString(number))
    }
}

struct NumberOrNumberStringVisitor;

impl<'de> de::Visitor<'de> for NumberOrNumberStringVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer, a decimal string, or a 0x-prepended hexadecimal string")
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

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::common::NomadIdentifier;

    use super::NumberOrNumberString;

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
        let five = NumberOrNumberString(5);
        let serialized = serde_json::to_value(&five).unwrap();

        let val = json! { 5 };
        assert_eq!(serialized, val);
        let n: NumberOrNumberString = serde_json::from_value(val).unwrap();
        assert_eq!(n, five);

        let val = json! { "5" };
        let n: NumberOrNumberString = serde_json::from_value(val).unwrap();
        assert_eq!(n, five);

        let val = json! { "0x5" };
        let n: NumberOrNumberString = serde_json::from_value(val).unwrap();
        assert_eq!(n, five);
    }
}
