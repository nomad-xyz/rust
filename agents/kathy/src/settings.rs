//! Configuration

use nomad_base::{decl_settings, AgentSecrets, AgentSettingsBlock};
use nomad_xyz_configuration::agent::kathy::ChatGenConfig;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KathySettingsBlock {
    pub interval: u64,
    pub chat: ChatGenConfig,
}

impl AgentSettingsBlock for KathySettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        _secrets: &AgentSecrets,
    ) -> Self {
        let config = &config.agent().get(home_network).unwrap().kathy;
        Self {
            interval: config.interval,
            chat: config.chat.clone(),
        }
    }
}

decl_settings!(Kathy, KathySettingsBlock,);
