#![allow(dead_code)] // TODO: Remove me

use crate::errors::Error;
use nomad_xyz_configuration::{agent::SignerConf, ChainConf, NomadConfig, TxSubmitterConf};
use std::{collections::HashMap, env, result::Result};

/// KillSwitch `Settings` contains all available configuration for all networks present
#[derive(Debug)]
pub(crate) struct Settings {
    /// NomadConfig
    pub config: NomadConfig,
    /// RPC endpoint configs
    pub rpcs: HashMap<String, ChainConf>,
    /// Transaction submission configs
    pub tx_submitters: HashMap<String, TxSubmitterConf>,
    /// Attestation signer configs
    pub attestation_signers: HashMap<String, SignerConf>,
}

impl Settings {
    /// Build new `Settings` from env and config file
    pub(crate) async fn new() -> Result<Self, Error> {
        // Get config
        let config = if let Ok(config_url) = env::var("CONFIG_URL") {
            NomadConfig::fetch(&config_url)
                .await
                .map_err(|_| Error::BadConfigVar(config_url.clone()))
        } else if let Ok(config_path) = env::var("CONFIG_PATH") {
            NomadConfig::from_file(&config_path)
                .map_err(|_| Error::BadConfigVar(config_path.clone()))
        } else {
            Err(Error::NoConfigVar)
        };
        let config = config?;

        // Load secrets manually instead of using `AgentSecrets::from_env` so we can load them
        // best effort instead of bailing on first error
        let networks = config.networks.clone();

        let rpcs = networks
            .iter()
            .filter_map(|n| ChainConf::from_env(&n.to_uppercase()).map(|conf| (n.clone(), conf)))
            .collect::<HashMap<_, _>>();

        let tx_submitters = networks
            .iter()
            .filter_map(|n| {
                TxSubmitterConf::from_env(&n.to_uppercase()).map(|conf| (n.clone(), conf))
            })
            .collect::<HashMap<_, _>>();

        // Load attestation signers for all networks explicitly using the form `<NETWORK>_ATTESTATION_SIGNER_ID`
        let attestation_signers = networks
            .iter()
            .filter_map(|n| {
                SignerConf::from_env(Some("ATTESTATION_SIGNER"), Some(&n.to_uppercase()))
                    .map(|conf| (n.clone(), conf))
            })
            .collect::<HashMap<_, _>>();

        Ok(Self {
            config,
            rpcs,
            tx_submitters,
            attestation_signers,
        })
    }
}
