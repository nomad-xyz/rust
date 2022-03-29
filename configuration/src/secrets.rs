//! Secrets configuration for agents.
//!
//! This struct is serialized from a JSON file or built drawing from a hosted
//! secrets manager backend. This struct is then used to finish building an
//! agents `Settings` block (see settings/mod.rs) along with a `NomadConfig`.
//!
//! Example JSON File Format
//! {
//!     "rpcs": {
//!         "ethereum": {
//!             "rpcStyle": "ethereum",
//!             "connection": {
//!                 "type": "http",
//!                 "url": ""
//!             }
//!         },
//!         "moonbeam": {
//!             "rpcStyle": "ethereum",
//!             "connection": {
//!                 "type": "http",
//!                 "url": ""
//!             }
//!         },
//!     },
//!     "transactionSigners": {
//!         "ethereum": {
//!             "type": "hexKey"
//!             "key": "",
//!         },
//!         "moonbeam": {
//!             "type": "hexKey"
//!             "key": "",
//!         },
//!     },
//!     "attestationSigner": {
//!         "key": "",
//!         "type": "hexKey"
//!     }
//! }

use crate::{agent::SignerConf, chains::ethereum, ChainConf, FromEnv};
use eyre::Result;
use serde::Deserialize;
use std::collections::HashMap;
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

    /// Ensure populated RPCs and transaction signers
    pub fn validate(&self, agent_name: &str) -> Result<()> {
        // TODO: replace agent name with associated type
        if agent_name == "updater" || agent_name == "watcher" {
            eyre::ensure!(
                self.attestation_signer.is_some(),
                "Must pass in attestation signer for {}",
                agent_name,
            )
        }

        for (network, chain_conf) in self.rpcs.iter() {
            match chain_conf {
                ChainConf::Ethereum(conn) => match conn {
                    ethereum::Connection::Http { url } => {
                        eyre::ensure!(!url.is_empty(), "Http url for {} empty!", network,);
                    }
                    ethereum::Connection::Ws { url } => {
                        eyre::ensure!(!url.is_empty(), "Ws url for {} empty!", network,);
                    }
                },
            }
        }

        for (network, signer_conf) in self.transaction_signers.iter() {
            match signer_conf {
                SignerConf::HexKey { key } => {
                    eyre::ensure!(
                        !key.as_ref().is_empty(),
                        "Hex signer key for {} empty!",
                        network,
                    );
                }
                SignerConf::Aws { id, region } => {
                    eyre::ensure!(!id.is_empty(), "ID for {} aws signer key empty!", network,);
                    eyre::ensure!(
                        !region.is_empty(),
                        "Region for {} aws signer key empty!",
                        network,
                    );
                }
                SignerConf::Node => (),
            }
        }

        Ok(())
    }
}

impl FromEnv for AgentSecrets {
    fn from_env(_prefix: &str) -> Option<Self> {
        let env = std::env::var("RUN_ENV").expect("missing RUN_ENV env var");
        let home = std::env::var("AGENT_HOME").expect("missing AGENT_HOME env var");

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
            let chain_conf = ChainConf::from_env(&format!("RPCS_{}", network_upper))
                .unwrap_or_else(|| panic!("missing info for {} ChainConf", network));
            let transaction_signer =
                SignerConf::from_env(&format!("TRANSACTION_SIGNERS_{}", network_upper))
                    .unwrap_or_else(|| panic!("missing info for {} SignerConf", network));

            secrets.rpcs.insert(network.to_owned(), chain_conf);
            secrets
                .transaction_signers
                .insert(network.to_owned(), transaction_signer);
        }

        let attestation_signer = SignerConf::from_env("ATTESTATION_SIGNER");
        secrets.attestation_signer = attestation_signer;

        Some(secrets)
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn it_builds_from_env() {

//     }
// }
