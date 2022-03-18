//! Configuration

use nomad_base::{decl_settings, AgentSecrets, AgentSettingsBlock};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RelayerSettingsBlock {
    pub interval: u64,
}

impl AgentSettingsBlock for RelayerSettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        _secrets: &AgentSecrets,
    ) -> Self {
        let interval = config.agent().get(home_network).unwrap().relayer.interval;
        Self { interval }
    }
}
decl_settings!(Relayer, RelayerSettingsBlock,);
