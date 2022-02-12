use std::collections::{HashMap, HashSet};

pub mod agent;
pub mod common;
pub mod contracts;
pub mod core_deploy;

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
