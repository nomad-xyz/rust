//! Configuration

use nomad_base::decl_settings;
use nomad_xyz_configuration::{agent::kathy::KathyConfig, FromEnv};

decl_settings!(Kathy, KathyConfig,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::NomadAgent;
    use nomad_xyz_configuration::AgentSecrets;

    #[test]
    fn it_builds_settings_from_env() {
        dotenv::from_filename("../../fixtures/env.test").unwrap();
        let run_env = dotenv::var("RUN_ENV").unwrap();
        let agent_home = dotenv::var("AGENT_HOME").unwrap();

        let settings = KathySettings::new().unwrap();

        let config = nomad_xyz_configuration::get_builtin(&run_env).unwrap();
        let secrets = AgentSecrets::from_env("").unwrap();

        settings
            .base
            .validate_against_config_and_secrets(
                crate::Kathy::AGENT_NAME,
                &agent_home,
                config,
                &secrets,
            )
            .unwrap();

        let agent_config = &config.agent().get("ethereum").unwrap().kathy;
        assert_eq!(settings.agent.interval, agent_config.interval);
        assert_eq!(settings.agent.chat, agent_config.chat);
    }
}
