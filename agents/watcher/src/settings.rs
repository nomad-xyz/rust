//! Configuration

use nomad_base::{
    decl_settings, AgentSecrets, AgentSettingsBlock, ChainSetup, ChainSetupType, SignerConf,
};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WatcherSettingsBlock {
    pub interval: u64,
    pub managers: HashMap<String, ChainSetup>,
    pub attestation_signer: SignerConf,
}

impl AgentSettingsBlock for WatcherSettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        secrets: &AgentSecrets,
    ) -> Self {
        let interval = config.agent().get(home_network).unwrap().watcher.interval;

        let home_connections = &config
            .protocol()
            .networks
            .get(home_network)
            .expect("!networks")
            .connections;

        // Connection manager has one xapp for every home connection
        let managers: HashMap<String, ChainSetup> = home_connections
            .iter()
            .map(|remote_network| {
                (
                    remote_network.to_owned(),
                    ChainSetup::from_nomad_config(
                        ChainSetupType::ConnectionManager { remote_network },
                        config,
                    ),
                )
            })
            .collect();

        let attestation_signer = secrets.attestation_signer.clone();

        Self {
            interval,
            managers,
            attestation_signer,
        }
    }
}
decl_settings!(Watcher, WatcherSettingsBlock,);
