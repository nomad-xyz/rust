//! Common Nomad data structures used across various parts of the stack (configuration, SDK, agents)

mod error;
pub use error::*;

mod macros;
pub use macros::*;

mod utils;
pub use utils::*;

use color_eyre::{eyre::bail, Report};
use ethers::prelude::{Address, H256};
use serde::{de, Deserializer};
use std::{fmt, ops::DerefMut, str::FromStr};

/// A Hex String of length `N` representing bytes of length `N / 2`
#[derive(Debug, Clone, PartialEq)]
pub struct HexString<const N: usize>(String);

impl<const N: usize> AsRef<String> for HexString<N> {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl<const N: usize> HexString<N> {
    /// Instantiate a new HexString from any `AsRef<str>`. Tolerates 0x
    /// prefixing. A succesful instantiation will create an owned copy of the
    /// string.
    pub fn from_string<S: AsRef<str>>(candidate: S) -> Result<Self, Report> {
        let s = strip_0x_prefix(candidate.as_ref());

        if s.len() != N {
            bail!("Expected string of length {}, got {}", N, s.len());
        }

        // Lazy. Should do the check as a cheaper action
        #[allow(clippy::question_mark)]
        if hex::decode(s).is_err() {
            bail!("String is not hex");
        }
        Ok(Self(s.to_owned()))
    }
}

impl<const N: usize> FromStr for HexString<N> {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

impl<'de, const N: usize> serde::Deserialize<'de> for HexString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_string(s).map_err(serde::de::Error::custom)
    }
}

/// A 32-byte network-agnostic identifier
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, serde::Serialize, Default, Hash)]
pub struct NomadIdentifier(H256);

impl std::fmt::Display for NomadIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl<'de> serde::Deserialize<'de> for NomadIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NomadIdentifierVisitor)
    }
}

impl std::ops::Deref for NomadIdentifier {
    type Target = H256;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NomadIdentifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

impl AsRef<[u8]> for NomadIdentifier {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsMut<[u8]> for NomadIdentifier {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl From<NomadIdentifier> for H256 {
    fn from(addr: NomadIdentifier) -> Self {
        addr.0
    }
}

impl From<NomadIdentifier> for [u8; 32] {
    fn from(addr: NomadIdentifier) -> Self {
        addr.0.into()
    }
}

impl NomadIdentifier {
    /// Check if the identifier is an ethereum address. This checks
    /// that the first 12 bytes are all 0.
    pub fn is_ethereum_address(&self) -> bool {
        self.0.as_bytes()[0..12].iter().all(|b| *b == 0)
    }

    /// Convert to an ethereum address. Return `None` if the conversion would
    /// drop non-0 bytes
    pub fn as_ethereum_address(&self) -> Result<Address, NomadTypeError> {
        let buf = self.as_fixed_bytes();
        if buf.starts_with(&[0u8; 12]) {
            Ok(Address::from_slice(&buf[12..]))
        } else {
            Err(NomadTypeError::AddressConversionError(*self))
        }
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

// Implement deser_nomad_number for all uint types
impl_deser_nomad_number!(u128, u64, u32, u16, u8);

/// An abstraction for allowing domains to be referenced by name or number
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum NameOrDomain {
    /// Domain name
    Name(String),
    /// Domain number
    Domain(u32),
}

impl From<String> for NameOrDomain {
    fn from(s: String) -> Self {
        Self::Name(s)
    }
}

impl From<u32> for NameOrDomain {
    fn from(t: u32) -> Self {
        Self::Domain(t)
    }
}

/// Domain/Address pair
#[derive(
    Default, Debug, Clone, Copy, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct NomadLocator {
    /// The domain
    pub domain: u32,
    /// The identifier on that domain
    pub id: NomadIdentifier,
}

/// An EVM beacon proxy
#[derive(
    Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash,
)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    /// Implementation address
    pub implementation: NomadIdentifier,
    /// Proxy address
    pub proxy: NomadIdentifier,
    /// Beacon address
    pub beacon: NomadIdentifier,
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
        // u64
        let five: u64 = 5;
        let serialized = serde_json::to_value(&five).unwrap();

        let val = json! { 5 };
        assert_eq!(serialized, val);
        let n = deser_nomad_u64(val).unwrap();
        assert_eq!(n, five);

        let val = json! { "5" };
        let n = deser_nomad_u64(val).unwrap();
        assert_eq!(n, five);

        let val = json! { "0x5" };
        let n = deser_nomad_u64(val).unwrap();
        assert_eq!(n, five);

        // u32
        let five: u32 = 5;
        let serialized = serde_json::to_value(&five).unwrap();

        let val = json! { 5 };
        assert_eq!(serialized, val);
        let n = deser_nomad_u32(val).unwrap();
        assert_eq!(n, five);

        let val = json! { "5" };
        let n = deser_nomad_u32(val).unwrap();
        assert_eq!(n, five);

        let val = json! { "0x5" };
        let n = deser_nomad_u32(val).unwrap();
        assert_eq!(n, five);
    }
}
