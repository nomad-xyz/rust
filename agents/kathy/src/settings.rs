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

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::NomadAgent;

    const RUN_ENV: &str = "test";
    const AGENT_HOME: &str = "ethereum";
    const SECRETS_PATH: &str = "../../fixtures/secrets.json";

    #[test]
    fn it_builds_settings_from_config_and_secrets() {
        std::env::set_var("RUN_ENV", RUN_ENV);
        std::env::set_var("AGENT_HOME", AGENT_HOME);
        std::env::set_var("SECRETS_PATH", SECRETS_PATH);

        let settings = KathySettings::new().unwrap();

        let config = nomad_xyz_configuration::get_builtin(RUN_ENV).unwrap();
        let secrets = AgentSecrets::from_file(SECRETS_PATH.into()).unwrap();

        settings
            .base
            .validate_against_config_and_secrets(
                crate::Kathy::AGENT_NAME,
                AGENT_HOME,
                config,
                &secrets,
            )
            .unwrap();

        let agent_config = &config.agent().get("ethereum").unwrap().kathy;
        assert_eq!(settings.agent.interval, agent_config.interval);
        assert_eq!(settings.agent.chat, agent_config.chat);
    }
}
