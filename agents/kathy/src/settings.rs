//! Configuration

use nomad_base::decl_settings;
use nomad_xyz_configuration::agent::kathy::KathyConfig;

decl_settings!(Kathy, KathyConfig,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::{AgentSecrets, NomadAgent};

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
        let secrets = AgentSecrets::from_file(SECRETS_PATH).unwrap();

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
