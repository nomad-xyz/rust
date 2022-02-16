//! Core deploy information

use std::collections::{HashMap, HashSet};

use crate::{
    agent::AgentConfig,
    common::{deser_nomad_number, NameOrDomain, NomadIdentifier, NomadLocator},
    contracts::CoreContracts,
};

/// Governance details
#[derive(
    Default, Debug, Clone, Copy, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Governance {
    /// Address of the recovery manager on this domain
    pub recovery_manager: NomadIdentifier,
    /// Length of the recovery timelock (in seconds) on this domain
    #[serde(deserialize_with = "deser_nomad_number")]
    pub recovery_timelock: u64,
}

/// Core network information
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreNetwork {
    /// Network name
    pub name: String,
    /// Network domain identifier
    #[serde(deserialize_with = "deser_nomad_number")]
    pub domain: u64,
    /// List of connections to other networks
    pub connections: HashSet<String>,
    /// Contract addresses
    pub contracts: CoreContracts,
    /// Governance info
    pub governance: Governance,
    /// List of updaters for this network
    pub updaters: HashSet<NomadIdentifier>,
    /// List of watchers for this network
    pub watchers: HashSet<NomadIdentifier>,
    /// Agent configuration for this network
    pub agents: AgentConfig,
}

impl CoreNetwork {
    /// Find the replica of a home on this network
    pub fn replica_of(&self, home_network: &str) -> Option<NomadIdentifier> {
        self.contracts.replica_of(home_network)
    }
}

/// Core deployment info
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDeploy {
    /// The domain and ID of the governor
    pub governor: NomadLocator,
    /// The network information for each network
    pub networks: HashMap<String, CoreNetwork>,
}

impl CoreDeploy {
    /// Resolve a `NameOrDomain` to a string, if that name/domain is present in this config
    pub fn resolve_domain(&self, domain: NameOrDomain) -> Option<String> {
        match domain {
            NameOrDomain::Name(name) => self.networks.get(&name).map(|_| name.to_owned()),
            NameOrDomain::Domain(number) => self
                .networks
                .iter()
                .find(|(_, net)| net.domain == number as u64)
                .map(|(net, _)| net.to_owned()),
        }
    }

    /// Get the network associated with the domain if any
    pub fn get_network(&self, domain: NameOrDomain) -> Option<&CoreNetwork> {
        self.resolve_domain(domain)
            .and_then(|name| self.networks.get(&name))
    }

    /// Returns a deploy containing ONLY the networks directly connected to the
    /// specified network
    pub fn trim_for_network(&self, network: &str) -> eyre::Result<CoreDeploy> {
        let core = self.networks.get(network).ok_or_else(|| {
            eyre::eyre!(
                "Could not trim for network {}. Network not found in config.",
                network
            )
        })?;

        let mut trimmed = self.clone();

        trimmed.networks = trimmed
            .networks
            .into_iter()
            .filter(|(k, _)| core.connections.contains(k))
            .collect();

        Ok(trimmed)
    }

    /// Returns a set of networks known to this core deploy
    pub fn networks(&self) -> HashSet<String> {
        self.networks.keys().map(ToOwned::to_owned).collect()
    }
}
