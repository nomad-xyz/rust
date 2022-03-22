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

        let settings = RelayerSettings::new().unwrap();

        let config = nomad_xyz_configuration::get_builtin(RUN_ENV).unwrap();
        let secrets = AgentSecrets::from_file(SECRETS_PATH).unwrap();

        settings
            .base
            .validate_against_config_and_secrets(
                crate::Relayer::AGENT_NAME,
                AGENT_HOME,
                config,
                &secrets,
            )
            .unwrap();

        let interval = config.agent().get("ethereum").unwrap().relayer.interval;
        assert_eq!(settings.agent.interval, interval);
    }
}
