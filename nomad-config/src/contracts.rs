//! Nomad Contract location configuration

use std::collections::HashMap;

use crate::common::NomadIdentifier;

/// An EVM beacon proxy
#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    /// Implementation address
    pub implementation: NomadIdentifier,
    /// Proxy address
    pub proxy: NomadIdentifier,
    /// Beacon address
    pub beacon: NomadIdentifier,
}

/// Evm Core Contracts
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmCoreContracts {
    /// UBC address
    pub upgrade_beacon_controller: NomadIdentifier,
    /// XApp Connection Manager address
    pub x_app_connection_manager: NomadIdentifier,
    /// Updater Manager address
    pub updater_manager: NomadIdentifier,
    /// Home Proxy details
    pub home: Proxy,
    /// Replica proxy details. Note these are the EVM replicas of remote domain.
    /// These are not the remote replicas of this domain
    pub replicas: HashMap<String, Proxy>,
}

/// Core Contract abstract
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum CoreContracts {
    /// EVM Core
    Evm(EvmCoreContracts),
    // leaving open future things here
}

impl CoreContracts {
    /// Get an iterator over the replicas present in this deploy
    pub fn replicas(&self) -> impl Iterator<Item = &String> {
        match self {
            CoreContracts::Evm(contracts) => contracts.replicas.keys(),
        }
    }

    /// True if the contracts contain a replica of the specified network.
    pub fn has_replica(&self, name: &str) -> bool {
        match self {
            CoreContracts::Evm(contracts) => contracts.replicas.contains_key(name),
        }
    }

    /// Locate the replica of the specified network (if known)
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

/// EVM Bridge Contracts
#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmBridgeContracts {
    /// Bridge Route proxy
    pub bridge_router: Proxy,
    /// Token Registry proxy
    pub token_registry: Proxy,
    /// Bridge Token proxy
    pub bridge_token: Proxy,
    /// Eth Helper address
    #[serde(default)]
    pub eth_helper: Option<NomadIdentifier>,
}

/// Bridge contract abstraction
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum BridgeContracts {
    /// EVM Bridge Contracts
    Evm(EvmBridgeContracts),
    // leaving open future things here
}

impl Default for BridgeContracts {
    fn default() -> Self {
        BridgeContracts::Evm(Default::default())
    }
}
