//! Configuration

use nomad_base::{decl_settings, AgentSecrets, AgentSettingsBlock, ChainSetup, SignerConf};
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
        _secrets: &AgentSecrets,
    ) -> Self {
        let interval = config.agent().get(home_network).unwrap().watcher.interval;
        Self {
            interval,
            managers: Default::default(),
            attestation_signer: Default::default(),
        }
    }
}
decl_settings!(Watcher, WatcherSettingsBlock,);
