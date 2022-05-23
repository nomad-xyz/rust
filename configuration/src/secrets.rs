//! Secrets configuration for agents.
//!
//! This struct built from environment variables. It is used alongside a
//! NomadConfig to build an agents `Settings` block (see settings/mod.rs).

use crate::{agent::SignerConf, chains::ethereum, ChainConf, FromEnv};
use eyre::Result;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::{fs::File, io::BufReader, path::Path};

/// Agent secrets block
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentSecrets {
    /// RPC endpoints
    pub rpcs: HashMap<String, ChainConf>,
    /// Transaction signers
    pub transaction_signers: HashMap<String, SignerConf>,
    /// Attestation signers
    pub attestation_signer: Option<SignerConf>,
}

impl AgentSecrets {
    /// Get JSON file and deserialize into AgentSecrets
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let secrets = serde_json::from_reader(reader)?;
        Ok(secrets)
    }

    /// Build AgentSecrets from environment variables
    pub fn from_env(networks: &HashSet<String>) -> Option<Self> {
        let mut secrets = AgentSecrets::default();

        for network in networks.iter() {
            let network_upper = network.to_uppercase();

            let chain_conf =
                ChainConf::from_env(&format!("RPCS_{}", network_upper), Some("RPCS_DEFAULT"))?;

            let transaction_signer = SignerConf::from_env(
                &format!("TRANSACTIONSIGNERS_{}", network_upper),
                Some("TRANSACTIONSIGNERS_DEFAULT"),
            )?;

            secrets.rpcs.insert(network.to_owned(), chain_conf);
            secrets
                .transaction_signers
                .insert(network.to_owned(), transaction_signer);
        }

        let attestation_signer = SignerConf::from_env("ATTESTATION_SIGNER", None);
        secrets.attestation_signer = attestation_signer;

        Some(secrets)
    }

    /// Ensure populated RPCs and transaction signers
    pub fn validate(&self, agent_name: &str, networks: &HashSet<String>) -> Result<()> {
        // TODO: replace agent name with associated type
        if agent_name == "updater" || agent_name == "watcher" {
            eyre::ensure!(
                self.attestation_signer.is_some(),
                "Must pass in attestation signer for {}",
                agent_name,
            )
        }

        for network in networks.iter() {
            let chain_conf = self
                .rpcs
                .get(network)
                .unwrap_or_else(|| panic!("no chainconf for {}", network));
            match chain_conf {
                ChainConf::Ethereum(conn) => match conn {
                    ethereum::Connection::Http(url) => {
                        eyre::ensure!(!url.is_empty(), "Http url for {} empty!", network,);
                    }
                    ethereum::Connection::Ws(url) => {
                        eyre::ensure!(!url.is_empty(), "Ws url for {} empty!", network,);
                    }
                },
            }

            let signer_conf = self
                .transaction_signers
                .get(network)
                .unwrap_or_else(|| panic!("no signerconf for {}", network));
            match signer_conf {
                SignerConf::HexKey(key) => {
                    eyre::ensure!(
                        !key.as_ref().is_empty(),
                        "Hex signer key for {} empty!",
                        network,
                    );
                }
                SignerConf::Aws { id } => {
                    eyre::ensure!(!id.is_empty(), "ID for {} aws signer key empty!", network,);
                }
                SignerConf::Node => (),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ethereum::Connection;
    use nomad_test::test_utils;

    #[test]
    #[serial_test::serial]
    fn it_builds_from_env_mixed() {
        test_utils::run_test_with_env_sync("../fixtures/env.test-signer-mixed", move || {
            let networks = &crate::get_builtin("test").unwrap().networks;
            let secrets =
                AgentSecrets::from_env(networks).expect("Failed to load secrets from env");

            assert_eq!(
                *secrets.transaction_signers.get("moonbeam").unwrap(),
                SignerConf::Aws {
                    id: "moonbeam_id".into(),
                }
            );
            assert_eq!(
                *secrets.transaction_signers.get("ethereum").unwrap(),
                SignerConf::HexKey(
                    "0x1111111111111111111111111111111111111111111111111111111111111111"
                        .parse()
                        .unwrap()
                )
            );
            assert_eq!(
                *secrets.transaction_signers.get("evmos").unwrap(),
                SignerConf::Aws {
                    id: "default_id".into(),
                }
            );
            assert_eq!(
                *secrets.rpcs.get("moonbeam").unwrap(),
                ChainConf::Ethereum(Connection::Http("https://rpc.api.moonbeam.network".into()))
            );
            assert_eq!(
                *secrets.rpcs.get("ethereum").unwrap(),
                ChainConf::Ethereum(Connection::Http(
                    "https://main-light.eth.linkpool.io/".into()
                ))
            );
            assert_eq!(
                *secrets.rpcs.get("evmos").unwrap(),
                ChainConf::Ethereum(Connection::Http("https://eth.bd.evmos.org:8545".into()))
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn it_builds_from_env_default() {
        test_utils::run_test_with_env_sync("../fixtures/env.test-signer-default", move || {
            let networks = &crate::get_builtin("test").unwrap().networks;
            let secrets =
                AgentSecrets::from_env(networks).expect("Failed to load secrets from env");

            let default_config = SignerConf::Aws {
                id: "default_id".into(),
            };
            for (_, config) in &secrets.transaction_signers {
                assert_eq!(*config, default_config);
            }
            for (_, config) in &secrets.rpcs {
                assert!(matches!(*config, ChainConf::Ethereum { .. }));
            }
        });
    }

    #[test]
    #[serial_test::serial]
    fn it_builds_from_env() {
        test_utils::run_test_with_env_sync("../fixtures/env.test", move || {
            let networks = &crate::get_builtin("test").unwrap().networks;
            AgentSecrets::from_env(networks).expect("Failed to load secrets from env");
        });
    }

    #[test]
    fn it_builds_from_file() {
        AgentSecrets::from_file("../fixtures/test_secrets.json")
            .expect("Failed to load secrets from file");
    }
}
