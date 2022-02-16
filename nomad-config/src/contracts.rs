use std::collections::HashMap;

use crate::common::NomadIdentifier;

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub implementation: NomadIdentifier,
    pub proxy: NomadIdentifier,
    pub beacon: NomadIdentifier,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmCoreContracts {
    pub upgrade_beacon_controller: NomadIdentifier,
    pub x_app_connection_manager: NomadIdentifier,
    pub updater_manager: NomadIdentifier,
    pub home: Proxy,
    pub replicas: HashMap<String, Proxy>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum CoreContracts {
    Evm(EvmCoreContracts),
    // leaving open future things here
}

impl CoreContracts {
    pub fn replicas(&self) -> impl Iterator<Item = &String> {
        match self {
            CoreContracts::Evm(contracts) => contracts.replicas.keys(),
        }
    }

    pub fn has_replica(&self, name: &str) -> bool {
        match self {
            CoreContracts::Evm(contracts) => contracts.replicas.contains_key(name),
        }
    }

    pub fn replica_of(&self, home_network: &str) -> Option<NomadIdentifier> {
        match self {
            CoreContracts::Evm(contracts) => contracts.replicas.get(home_network).map(|n| n.proxy),
        }
    }
}

impl Default for CoreContracts {
    fn default() -> Self {
        CoreContracts::Evm(Default::default())
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmBridgeContracts {
    pub bridge_router: Proxy,
    pub token_registry: Proxy,
    pub bridge_token: Proxy,
    #[serde(default)]
    pub eth_helper: Option<NomadIdentifier>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum BridgeContracts {
    Evm(EvmBridgeContracts),
    // leaving open future things here
}

impl Default for BridgeContracts {
    fn default() -> Self {
        BridgeContracts::Evm(Default::default())
    }
}
