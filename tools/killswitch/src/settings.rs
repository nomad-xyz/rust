use color_eyre::{eyre, Result};
use nomad_xyz_configuration::{agent::SignerConf, ChainConf, NomadConfig, TxSubmitterConf};
use std::{collections::HashMap, env};

/// KillSwitchSettings contains all available configuration for all networks in config
#[derive(Debug)]
pub(crate) struct KillSwitchSettings {
    /// RPC endpoint configs
    pub rpcs: HashMap<String, Option<ChainConf>>,
    /// Transaction submission configs
    pub tx_submitters: HashMap<String, Option<TxSubmitterConf>>,
    /// Attestation signer configs
    pub attestation_signers: HashMap<String, Option<SignerConf>>,
}

impl KillSwitchSettings {
    /// Build new KillSwitchSettings from env and config file
    pub(crate) async fn new() -> Result<Self> {
        // Get config
        let config = if let Some(config_url) = env::var("CONFIG_URL").ok() {
            NomadConfig::fetch(&config_url)
                .await
                .expect(&format!("Unable to load config from {}", config_url))
        } else if let Some(config_path) = env::var("CONFIG_PATH").ok() {
            NomadConfig::from_file(&config_path)
                .expect(&format!("Unable to load config from {}", config_path))
        } else {
            eyre::bail!(
                "No configuration found. Set CONFIG_URL or CONFIG_PATH environment variable"
            )
        };

        // Load secrets manually instead of using `AgentSecrets::from_env` so we can load them
        // best effort instead of bailing on first error
        let networks = config.networks.clone();

        let rpcs = networks
            .iter()
            .map(|n| (n.clone(), ChainConf::from_env(&n.to_uppercase())))
            .collect::<HashMap<_, _>>();

        let tx_submitters = networks
            .iter()
            .map(|n| (n.clone(), TxSubmitterConf::from_env(&n.to_uppercase())))
            .collect::<HashMap<_, _>>();

        // Load attestation signers for all networks explicitly using the form `<NETWORK>_ATTESTATION_SIGNER_ID`
        let attestation_signers = networks
            .iter()
            .map(|n| {
                (
                    n.clone(),
                    SignerConf::from_env(Some("ATTESTATION_SIGNER"), Some(&n.to_uppercase())),
                )
            })
            .collect::<HashMap<_, _>>();

        return Ok(Self {
            rpcs,
            tx_submitters,
            attestation_signers,
        });
    }
}
