//! Core deploy information

use crate::bridge::BridgeConfiguration;
use nomad_types::{
    deser_nomad_u32, deser_nomad_u64, deser_nomad_u8, NameOrDomain, NomadIdentifier, NomadLocator,
};
use std::collections::{HashMap, HashSet};

/// Governance details
#[derive(
    Default, Debug, Clone, Copy, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct Governance {
    /// Address of the recovery manager on this domain
    pub recovery_manager: NomadIdentifier,
    /// Length of the recovery timelock (in seconds) on this domain
    #[serde(deserialize_with = "deser_nomad_u64")]
    pub recovery_timelock: u64,
}

/// Nomad Contract deploy-time config
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractConfig {
    /// Optimsitic seconds for replicas to wait
    #[serde(deserialize_with = "deser_nomad_u64")]
    pub optimistic_seconds: u64,
    /// Default process gas
    #[serde(deserialize_with = "deser_nomad_u64")]
    pub process_gas: u64,
    /// Reserve gas
    #[serde(deserialize_with = "deser_nomad_u64")]
    pub reserve_gas: u64,
    /// Maximum preflight gas
    #[serde(deserialize_with = "deser_nomad_u64")]
    pub maximum_gas: u64,
    /// List of updaters for this network
    pub updater: NomadIdentifier,
    /// List of watchers for this network
    pub watchers: HashSet<NomadIdentifier>,
    /// Governance info
    pub governance: Governance,
}

/// Core network information
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSpecs {
    /// EVM chain id. 0 for non-EVM chains
    #[serde(default, deserialize_with = "deser_nomad_u64")]
    pub chain_id: u64,
    /// Block time on the network
    #[serde(deserialize_with = "deser_nomad_u64")]
    pub block_time: u64,
    /// Timelag for agents using the timelag provider
    #[serde(deserialize_with = "deser_nomad_u8")]
    pub finalization_blocks: u8,
    /// True if the networks supports 1559. Otherwise false
    #[serde(default)]
    pub supports_1559: bool,
    /// Desired number of confirmations on transactions
    #[serde(deserialize_with = "deser_nomad_u64")]
    pub confirmations: u64,
    /// Block explorer URL
    pub block_explorer: String,
    /// Number of blocks to include in a page while indexing
    #[serde(deserialize_with = "deser_nomad_u32")]
    pub index_page_size: u32,
}

/// Specifier for deploy-time custom bridge tokens
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CustomTokenSpecifier {
    /// Token domain and id
    pub token: NomadLocator,
    /// Token name
    pub name: String,
    /// Token Symbol
    pub symbol: String,
    /// Token decimals
    pub decimals: u8,
}

/// Core network information
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Domain {
    /// Network name
    pub name: String,
    /// Network domain identifier
    #[serde(deserialize_with = "deser_nomad_u32")]
    pub domain: u32,
    /// List of connections to other networks
    pub connections: HashSet<String>,
    /// Nomad protocol configuration options
    pub configuration: ContractConfig,
    /// Network specifications
    pub specs: NetworkSpecs,
    /// Bridge contract configuration options
    pub bridge_configuration: BridgeConfiguration,
}

/// Core deployment info
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    /// The domain and ID of the governor
    pub governor: NomadLocator,
    /// The network information for each network
    pub networks: HashMap<String, Domain>,
}

impl NetworkInfo {
    /// Resolve a `NameOrDomain` to a string, if that name/domain is present in this config
    pub fn resolve_domain(&self, domain: NameOrDomain) -> Option<String> {
        match domain {
            NameOrDomain::Name(name) => self.networks.get(&name).map(|_| name.to_owned()),
            NameOrDomain::Domain(number) => self
                .networks
                .iter()
                .find(|(_, net)| net.domain == number)
                .map(|(net, _)| net.to_owned()),
        }
    }

    /// Get the network associated with the domain if any
    pub fn get_network(&self, domain: NameOrDomain) -> Option<&Domain> {
        self.resolve_domain(domain)
            .and_then(|name| self.networks.get(&name))
    }

    /// Returns a deploy containing ONLY the networks directly connected to the
    /// specified network
    pub fn trim_for_network(&self, network: &str) -> eyre::Result<NetworkInfo> {
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
