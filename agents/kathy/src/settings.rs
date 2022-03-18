//! Configuration

use nomad_base::{decl_settings, AgentSecrets, AgentSettingsBlock};
use nomad_types::agent::kathy::ChatGenConfig;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct KathySettingsBlock {
    pub interval: u64,
    pub chat: ChatGenConfig,
}

// TODO: add kathy settings to configuration
impl AgentSettingsBlock for KathySettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        _secrets: &AgentSecrets,
    ) -> Self {
        let interval = config.agent().get(home_network).unwrap().kathy.interval;
        Self {
            interval,
            chat: Default::default(),
        }
    }
}

decl_settings!(Kathy, KathySettingsBlock,);
