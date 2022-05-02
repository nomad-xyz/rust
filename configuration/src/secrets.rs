//! Secrets configuration for agents.
//!
//! This struct built from environment variables. It is used alongside a
//! NomadConfig to build an agents `Settings` block (see settings/mod.rs).

use crate::{agent::SignerConf, chains::ethereum, ChainConf, FromEnv, TxSubmitterConf};
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
            let chain_conf = ChainConf::from_env(&format!("RPCS_{}", network_upper))?;
            let transaction_signer =
                SignerConf::from_env(&format!("TRANSACTIONSIGNERS_{}", network_upper))?;

            secrets.rpcs.insert(network.to_owned(), chain_conf);
            secrets
                .transaction_signers
                .insert(network.to_owned(), transaction_signer);
        }

        let attestation_signer = SignerConf::from_env("ATTESTATION_SIGNER");
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

            let submitter_conf = self
                .tx_submitters
                .get(network)
                .unwrap_or_else(|| panic!("no signerconf for {}", network));
            match submitter_conf {
                TxSubmitterConf::Ethereum(conf) => match conf {
                    ethereum::TxSubmitterConf::Local(signer_conf) => {
                        signer_conf.validate(network)?
                    }
                    ethereum::TxSubmitterConf::Gelato(gelato_conf) => {
                        gelato_conf.signer.validate(network)?
                    }
                },
            };
        }

        Ok(())
    }
}

impl FromEnv for AgentSecrets {
    fn from_env(_prefix: &str) -> Option<Self> {
        let env = std::env::var("RUN_ENV").ok()?;
        let home = std::env::var("AGENT_HOME").ok()?;

        let config = crate::get_builtin(&env)
            .expect("couldn't retrieve config!")
            .to_owned();

        let mut networks = config
            .protocol()
            .networks
            .get(&home)
            .expect("!networks")
            .connections
            .to_owned();
        networks.insert(home);

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

        let attestation_signer = SignerConf::from_env("ATTESTATION_SIGNER");
        secrets.attestation_signer = attestation_signer;

        Some(secrets)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const SECRETS_JSON_PATH: &str = "../fixtures/test_secrets.json";
    const SECRETS_ENV_PATH: &str = "../fixtures/env.test";

    #[test]
    fn it_builds_from_env() {
        let networks = &crate::get_builtin("test").unwrap().networks;
        dotenv::from_filename(SECRETS_ENV_PATH).unwrap();
        AgentSecrets::from_env(networks).expect("Failed to load secrets from env");
    }

    #[test]
    fn it_builds_from_file() {
        AgentSecrets::from_file(SECRETS_JSON_PATH).expect("Failed to load secrets from file");
    }
}
