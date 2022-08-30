//! Secrets configuration for agents.
//!
//! This struct built from environment variables. It is used alongside a
//! NomadConfig to build an agents `Settings` block (see settings/mod.rs).

use crate::{
    agent::SignerConf,
    chains::{ethereum, substrate},
    ChainConf, Connection, TxSubmitterConf,
};
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
    /// Transaction submission variants
    pub tx_submitters: HashMap<String, TxSubmitterConf>,
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

            let chain_conf = ChainConf::from_env(&network_upper)?;
            let tx_submitter = TxSubmitterConf::from_env(&network_upper)?;

            secrets.rpcs.insert(network.to_owned(), chain_conf);
            secrets
                .tx_submitters
                .insert(network.to_owned(), tx_submitter);
        }

        let attestation_signer = SignerConf::from_env(Some("ATTESTATION_SIGNER"), None);
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

            let conn = match chain_conf {
                ChainConf::Ethereum(conn) => conn,
                ChainConf::Substrate(conn) => conn,
            };
            match conn {
                Connection::Http(url) => {
                    eyre::ensure!(!url.is_empty(), "Http url for {} empty!", network,);
                }
                Connection::Ws(url) => {
                    eyre::ensure!(!url.is_empty(), "Ws url for {} empty!", network,);
                }
            }

            let submitter_conf = self
                .tx_submitters
                .get(network)
                .unwrap_or_else(|| panic!("no submitter conf for {}", network));
            match submitter_conf {
                TxSubmitterConf::Ethereum(conf) => match conf {
                    ethereum::TxSubmitterConf::Local(signer_conf) => {
                        signer_conf.validate(network)?
                    }
                    ethereum::TxSubmitterConf::Gelato(gelato_conf) => {
                        gelato_conf.sponsor.validate(network)?
                    }
                },
                TxSubmitterConf::Substrate(conf) => match conf {
                    substrate::TxSubmitterConf::Local(signer_conf) => {
                        signer_conf.validate(network)?
                    }
                },
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ethereum::GelatoConf, NomadConfig};
    use nomad_test::test_utils;

    #[test]
    #[serial_test::serial]
    fn it_builds_from_env_local_mixed() {
        test_utils::run_test_with_env_sync("../fixtures/env.test-local-signer-mixed", move || {
            let networks = &crate::get_builtin("test").unwrap().networks;
            let secrets =
                AgentSecrets::from_env(networks).expect("Failed to load secrets from env");

            assert_eq!(
                *secrets.tx_submitters.get("moonbeam").unwrap(),
                TxSubmitterConf::Ethereum(ethereum::TxSubmitterConf::Local(SignerConf::Aws {
                    id: "moonbeam_id".into(),
                }))
            );
            assert_eq!(
                *secrets.tx_submitters.get("ethereum").unwrap(),
                TxSubmitterConf::Ethereum(ethereum::TxSubmitterConf::Local(SignerConf::HexKey(
                    "0x1111111111111111111111111111111111111111111111111111111111111111"
                        .parse()
                        .unwrap()
                )))
            );
            assert_eq!(
                *secrets.tx_submitters.get("evmos").unwrap(),
                TxSubmitterConf::Ethereum(ethereum::TxSubmitterConf::Local(SignerConf::Aws {
                    id: "default_id".into(),
                }))
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
    fn it_builds_from_env_local_default() {
        test_utils::run_test_with_env_sync(
            "../fixtures/env.test-local-signer-default",
            move || {
                let networks = &crate::get_builtin("test").unwrap().networks;
                let secrets =
                    AgentSecrets::from_env(networks).expect("Failed to load secrets from env");

                let default_config_ethereum = ethereum::TxSubmitterConf::Local(SignerConf::Aws {
                    id: "default_id".into(),
                });
                for config in secrets.tx_submitters.values() {
                    match config {
                        TxSubmitterConf::Ethereum(conf) => {
                            assert_eq!(*conf, default_config_ethereum)
                        }
                        // Default set to ethereum, not checking substrate variant
                        TxSubmitterConf::Substrate(_) => continue,
                    }
                }
            },
        );
    }

    #[test]
    #[serial_test::serial]
    fn it_builds_from_env_gelato_default() {
        test_utils::run_test_with_env_sync("../fixtures/env.test-gelato-default", move || {
            let networks = &crate::get_builtin("test").unwrap().networks;
            let secrets =
                AgentSecrets::from_env(networks).expect("Failed to load secrets from env");

            let default_config_ethereum = ethereum::TxSubmitterConf::Gelato(GelatoConf {
                sponsor: SignerConf::Aws {
                    id: "default_id".into(),
                },
                fee_token: "0x1234".to_owned(),
            });
            for config in secrets.tx_submitters.values() {
                match config {
                    TxSubmitterConf::Ethereum(conf) => assert_eq!(*conf, default_config_ethereum),
                    // Default set to ethereum, not checking substrate variant
                    TxSubmitterConf::Substrate(_) => continue,
                }
            }
        });
    }

    #[test]
    #[serial_test::serial]
    fn it_builds_test_config_from_env() {
        test_utils::run_test_with_env_sync("../fixtures/env.test", move || {
            let networks = &crate::get_builtin("test").unwrap().networks;
            let secrets =
                AgentSecrets::from_env(networks).expect("Failed to load secrets from env");
            secrets
                .validate("", networks)
                .expect("Failed to validate secrets");
        });
    }

    #[test]
    fn it_builds_test_config_from_file() {
        let networks = &crate::get_builtin("test").unwrap().networks;
        let secrets = AgentSecrets::from_file("../fixtures/test_secrets.json")
            .expect("Failed to load secrets from file");
        secrets
            .validate("", networks)
            .expect("Failed to validate secrets");
    }

    #[test]
    #[serial_test::serial]
    fn it_builds_multi_vm_config_from_env() {
        test_utils::run_test_with_env_sync("../fixtures/env.test-multi-vm", move || {
            let networks = NomadConfig::from_file("configs/testMultiVm.json")
                .unwrap()
                .networks;
            let secrets =
                AgentSecrets::from_env(&networks).expect("Failed to load secrets from env");
            secrets
                .validate("", &networks)
                .expect("Failed to validate secrets");
        });
    }

    #[test]
    fn it_builds_multi_vm_config_from_file() {
        let networks = NomadConfig::from_file("configs/testMultiVm.json")
            .unwrap()
            .networks;
        let secrets = AgentSecrets::from_file("../fixtures/test_multi_vm_secrets.json")
            .expect("Failed to load secrets from file");
        secrets
            .validate("", &networks)
            .expect("Failed to validate secrets");
    }
}
