//! Configuration

use nomad_base::decl_settings;
use nomad_xyz_configuration::agent::kathy::KathyConfig;

decl_settings!(Kathy, KathyConfig,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::NomadAgent;
    use nomad_test::test_utils;
    use nomad_xyz_configuration::AgentSecrets;

    #[tokio::test]
    #[serial_test::serial]
    async fn it_builds_settings_from_env() {
        test_utils::run_test_with_env("../../fixtures/env.test", || async move {
            let run_env = dotenv::var("RUN_ENV").unwrap();
            let agent_home = dotenv::var("AGENT_HOME").unwrap();

            let env_vars = std::env::vars();
            for (key, value) in env_vars.into_iter() {
                println!("{} = {:?}", key, value);
            }

            let settings = KathySettings::new().unwrap();
            println!("Settings: {:?}", settings);

            let config = nomad_xyz_configuration::get_builtin(&run_env).unwrap();
            let secrets = AgentSecrets::from_env(&config.networks).unwrap();

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
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_builds_settings_from_external_file() {
        test_utils::run_test_with_env("../../fixtures/env.external", || async move {
            std::env::set_var("CONFIG_PATH", "../../fixtures/external_config.json");
            let agent_home = dotenv::var("AGENT_HOME").unwrap();

            let settings = KathySettings::new().unwrap();

            let config = nomad_xyz_configuration::NomadConfig::from_file(
                "../../fixtures/external_config.json",
            )
            .unwrap();
            let secrets = AgentSecrets::from_env(&config.networks).unwrap();

            settings
                .base
                .validate_against_config_and_secrets(
                    crate::Kathy::AGENT_NAME,
                    &agent_home,
                    &config,
                    &secrets,
                )
                .unwrap();

            let agent_config = &config.agent().get("ethereum").unwrap().kathy;
            assert_eq!(settings.agent.interval, agent_config.interval);
            assert_eq!(settings.agent.chat, agent_config.chat);
        })
        .await
    }
}
