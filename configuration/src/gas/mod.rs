//! Per-chain gas configurations

use serde::{de, Deserialize, Serialize};
use std::{convert::Infallible, fmt, str::FromStr};

mod defaults;
use defaults::EVM_DEFAULT;

/// Gas configuration for core and bridge contract methods
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NomadGasConfig {
    /// Core gas limits
    pub core: CoreGasConfig,
    /// Bridge gas limits
    pub bridge: BridgeGasConfig,
}

/// Gas configuration for core contract methods
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoreGasConfig {
    /// Home gas limits
    pub home: HomeGasLimits,
    /// Replica gas limits
    pub replica: ReplicaGasLimits,
    /// Connection manager gas limits
    pub connection_manager: ConnectionManagerGasLimits,
}

/// Gas limits specifically for a home update call
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeUpdateGasLimit {
    /// Per message additional gas cost
    pub per_message: u64,
    /// Base gas limits
    pub base: u64,
}

/// Home gas limits
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HomeGasLimits {
    /// Update
    pub update: HomeUpdateGasLimit,
    /// Improper update
    pub improper_update: HomeUpdateGasLimit,
    /// Double update
    pub double_update: u64,
}

/// Replica gas limits
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaGasLimits {
    /// Update
    pub update: u64,
    /// Prove
    pub prove: u64,
    /// Process
    pub process: u64,
    /// Prove and process
    pub prove_and_process: u64,
    /// Double update
    pub double_update: u64,
}

/// Connection manager gas limits
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionManagerGasLimits {
    /// Owner unenroll replica
    pub owner_unenroll_replica: u64,
    /// Unenroll replica
    pub unenroll_replica: u64,
}

/// Gas configuration for bridge contract methods
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BridgeGasConfig {
    /// BridgeRouter gas limits
    pub bridge_router: BridgeRouterGasLimits,
    /// EthHelper gas limits
    pub eth_helper: EthHelperGasLimits,
}

/// Gas limits for BridgeRouter
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BridgeRouterGasLimits {
    /// Send
    pub send: u64,
}

/// Gas limits for EthHelper
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EthHelperGasLimits {
    /// Send
    pub send: u64,
    /// Send to EVM like
    pub send_to_evm_like: u64,
}

pub(crate) mod gas_map_ser {
    use serde::Deserializer;
    use std::collections::HashMap;

    use super::*;

    /// A convenience struct for intermediate deser of gas configs
    #[derive(Debug, Copy, Clone, Serialize, PartialEq)]
    pub(crate) struct NomadGasConfigInternal(NomadGasConfig);

    impl From<NomadGasConfig> for NomadGasConfigInternal {
        fn from(conf: NomadGasConfig) -> Self {
            NomadGasConfigInternal(conf)
        }
    }

    impl std::ops::Deref for NomadGasConfigInternal {
        type Target = NomadGasConfig;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<'de> Deserialize<'de> for NomadGasConfigInternal {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(NomadGasConfigInternalVisitor)
        }
    }

    impl FromStr for NomadGasConfigInternal {
        type Err = Infallible;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s == "evmDefault" {
                Ok(NomadGasConfigInternal(EVM_DEFAULT))
            } else {
                panic!("Unrecognized string variant for gas config")
            }
        }
    }

    struct NomadGasConfigInternalVisitor;
    impl<'de> de::Visitor<'de> for NomadGasConfigInternalVisitor {
        type Value = NomadGasConfigInternal;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a gas config map or gas config default")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(v).unwrap())
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            Ok(NomadGasConfigInternal(NomadGasConfig::deserialize(
                de::value::MapAccessDeserializer::new(map),
            )?))
        }
    }

    pub(crate) fn deserialize<'de, D>(d: D) -> Result<HashMap<String, NomadGasConfig>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, NomadGasConfigInternal>::deserialize(d)?;

        Ok(map.into_iter().map(|(k, v)| (k, *v)).collect())
    }
}
