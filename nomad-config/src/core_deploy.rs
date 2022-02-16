use std::collections::{HashMap, HashSet};

use crate::{
    agent::AgentConfig,
    common::{deser_nomad_number, NameOrDomain, NomadIdentifier},
    contracts::CoreContracts,
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Governor {
    pub address: NomadIdentifier,
    #[serde(deserialize_with = "deser_nomad_number")]
    pub domain: u64,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Governance {
    pub recovery_manager: NomadIdentifier,
    #[serde(deserialize_with = "deser_nomad_number")]
    pub recovery_timelock: u64,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreNetwork {
    pub name: String,
    #[serde(deserialize_with = "deser_nomad_number")]
    pub domain: u64,
    pub connections: Vec<String>,
    pub contracts: CoreContracts,
    pub governance: Governance,
    pub updaters: Vec<NomadIdentifier>,
    pub watchers: Vec<NomadIdentifier>,
    pub agents: AgentConfig,
}

impl CoreNetwork {
    pub fn replica_of(&self, home_network: &str) -> Option<NomadIdentifier> {
        self.contracts.replica_of(home_network)
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDeploy {
    pub governor: Governor,
    pub networks: HashMap<String, CoreNetwork>,
}

impl CoreDeploy {
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
