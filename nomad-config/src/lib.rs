use std::collections::{HashMap, HashSet};

pub mod agent;
pub mod common;
pub mod contracts;
pub mod core_deploy;

use common::{NameOrDomain, NomadIdentifier};
use contracts::BridgeContracts;
use core_deploy::CoreDeploy;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NomadConfig {
    pub environment: String,
    pub networks: HashSet<String>,
    pub rpcs: HashMap<String, HashSet<String>>,
    pub core: CoreDeploy,
    pub bridge: HashMap<String, BridgeContracts>,
}

impl NomadConfig {
    /// Resolve a name or domain
    pub fn resolve_domain(&self, domain: NameOrDomain) -> Option<String> {
        self.core.resolve_domain(domain)
    }

    /// Syntactcially validate the config, consuming and returning self
    pub fn chained_validate(self) -> eyre::Result<Self> {
        self.validate()?;
        Ok(self)
    }

    /// Syntactically validate the config
    pub fn validate(&self) -> eyre::Result<()> {
        // check core and bridge exist for all listed networks
        for network in self.networks.iter() {
            eyre::ensure!(
                self.core.networks.contains_key(network),
                "Core for network named {} not found.",
                network
            );
            eyre::ensure!(
                self.bridge.contains_key(network),
                "Bridge for network named {} not found.",
                network
            )
        }

        // check each core contains replicas ONLY for its listed connections
        for (name, network) in self.core.networks.iter() {
            eyre::ensure!(
                self.networks.contains(name),
                "Core named {} not found in configured networks",
                name,
            );

            for connection in network.connections.iter() {
                eyre::ensure!(
                    network.contracts.has_replica(connection),
                    "Replica named {} not found on core named {} despite being listed in core connections",
                    connection,
                    name,
                );
            }

            for replica in network.contracts.replicas() {
                eyre::ensure!(
                    network.connections.contains(replica),
                    "Replica named {} on core named {} not found in core's configured connections",
                    replica,
                    name
                );
                eyre::ensure!(
                    self.networks.contains(name),
                    "Replica named {} on core named {} not found in base config's configured networks",
                    replica,
                    name,
                );
            }
        }

        // check that no extra bridges are listed
        for network in self.bridge.keys() {
            eyre::ensure!(
                self.networks.contains(network),
                "Bridge named {} not found in configured networks",
                network,
            );
        }

        Ok(())
    }

    /// Returns a config containing ONLY the networks directly connected to the
    /// specified network. This should be used for agent bootup
    pub fn trim_to_network(&self, network: impl AsRef<str>) -> eyre::Result<NomadConfig> {
        let network = network.as_ref();
        let mut trimmed = self.clone();
        trimmed.core = trimmed.core.trim_for_network(network)?;
        trimmed.networks = trimmed.core.networks();
        trimmed.bridge = trimmed
            .bridge
            .into_iter()
            .filter(|(k, _)| trimmed.networks.contains(k))
            .collect();

        Ok(trimmed)
    }

    /// Find the replica of home_network on target_network
    pub fn locate_replica_of(
        &self,
        home_network: NameOrDomain,
        target_network: NameOrDomain,
    ) -> Option<NomadIdentifier> {
        let home_network = self.resolve_domain(home_network)?;
        self.core
            .get_network(target_network)
            .and_then(|n| n.replica_of(&home_network))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_loads_the_sample_config() {
        let _config: NomadConfig =
            serde_json::from_reader(std::fs::File::open("./test.json").unwrap()).unwrap();
        dbg!(&_config);
    }

    #[test]
    fn it_allows_default_config() {
        dbg!(NomadConfig::default());
    }
}
