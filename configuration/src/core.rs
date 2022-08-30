//! Nomad Contract location configuration

use std::collections::HashMap;

use nomad_types::deser_nomad_u32;
use nomad_types::{NomadIdentifier, Proxy};

/// Evm Core Contracts
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthereumCoreDeploymentInfo {
    /// Contract Deploy Height
    #[serde(default, deserialize_with = "deser_nomad_u32")]
    pub deploy_height: u32,
    /// UBC address
    pub upgrade_beacon_controller: NomadIdentifier,
    /// XApp Connection Manager address
    pub x_app_connection_manager: NomadIdentifier,
    /// Updater Manager address
    pub updater_manager: NomadIdentifier,
    /// Governance router proxy details
    pub governance_router: Proxy,
    /// Home Proxy details
    pub home: Proxy,
    /// Replica proxy details. Note these are the EVM replicas of remote domain.
    /// These are not the remote replicas of this domain
    pub replicas: HashMap<String, Proxy>,
}

/// Empty Substrate contracts
#[derive(Default, Copy, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubstrateCoreDeploymentInfo {
    /// Contract Deploy Height
    #[serde(default, deserialize_with = "deser_nomad_u32")]
    pub deploy_height: u32,
    // TODO: add replicas for substrate rollout v2
}

/// Core Contract abstract
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum CoreDeploymentInfo {
    /// EVM Core
    Ethereum(EthereumCoreDeploymentInfo),
    /// Substrate core
    Substrate(SubstrateCoreDeploymentInfo),
}

impl CoreDeploymentInfo {
    /// Get an iterator over the replicas present in this deploy
    pub fn replicas(&self) -> impl Iterator<Item = &String> {
        match self {
            CoreDeploymentInfo::Ethereum(contracts) => contracts.replicas.keys(),
            CoreDeploymentInfo::Substrate(_) => {
                unimplemented!("Replicas do not exist in Substrate implementations")
            }
        }
    }

    /// True if the contracts contain a replica of the specified network.
    pub fn has_replica(&self, name: &str) -> bool {
        match self {
            CoreDeploymentInfo::Ethereum(contracts) => contracts.replicas.contains_key(name),
            CoreDeploymentInfo::Substrate(_) => {
                unimplemented!("Replicas do not exist in Substrate implementations")
            }
        }
    }

    /// Locate the replica of the specified network (if known)
    pub fn replica_of(&self, home_network: &str) -> Option<NomadIdentifier> {
        match self {
            CoreDeploymentInfo::Ethereum(contracts) => {
                contracts.replicas.get(home_network).map(|n| n.proxy)
            }
            CoreDeploymentInfo::Substrate(_) => {
                unimplemented!("Replicas do not exist in Substrate implementations")
            }
        }
    }
}

impl Default for CoreDeploymentInfo {
    fn default() -> Self {
        CoreDeploymentInfo::Ethereum(Default::default())
    }
}
