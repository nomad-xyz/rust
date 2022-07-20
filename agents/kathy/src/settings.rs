//! Configuration

use nomad_base::decl_settings;
use nomad_xyz_configuration::agent::kathy::KathyConfig;

decl_settings!(Kathy, KathyConfig,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::{get_remotes_from_env, NomadAgent};
    use nomad_test::test_utils;
    use nomad_xyz_configuration::{agent::kathy::ChatGenConfig, AgentSecrets};
    use serde_json::json;
    use std::collections::HashSet;

    #[tokio::test]
    #[serial_test::serial]
    async fn it_loads_remote_config() {
        let config =
            nomad_xyz_configuration::NomadConfig::from_file("../../fixtures/external_config.json")
                .unwrap();
        let config_json = json!(config).to_string();
        test_utils::run_test_with_http_response(config_json, |url| async move {
            test_utils::run_test_with_env("../../fixtures/env.external", || async move {
                std::env::set_var("CONFIG_URL", url);

                let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

                let settings = KathySettings::new().await.unwrap();

                let remotes = get_remotes_from_env!(agent_home, config);
                let mut networks = remotes.clone();
                networks.insert(agent_home.clone());

                let secrets = AgentSecrets::from_env(&networks).unwrap();
                secrets
                    .validate("kathy", &networks)
                    .expect("!secrets validate");

                settings
                    .base
                    .validate_against_config_and_secrets(
                        crate::Kathy::AGENT_NAME,
                        &agent_home,
                        &remotes,
                        &config,
                        &secrets,
                    )
                    .unwrap();

                let mut settings_networks = settings
                    .base
                    .replicas
                    .values()
                    .map(|c| c.name.clone())
                    .collect::<HashSet<_>>();
                settings_networks.insert(settings.base.home.name.clone());

                assert_eq!(config.networks, settings_networks);
            })
            .await
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_overrides_settings_from_env() {
        test_utils::run_test_with_env("../../fixtures/env.test-agents", || async move {
            let run_env = dotenv::var("RUN_ENV").unwrap();
            let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

            let settings = KathySettings::new().await.unwrap();

            let config = nomad_xyz_configuration::get_builtin(&run_env).unwrap();

            let remotes = get_remotes_from_env!(agent_home, config);
            let mut networks = remotes.clone();
            networks.insert(agent_home.clone());

            let secrets = AgentSecrets::from_env(&networks).unwrap();

            settings
                .base
                .validate_against_config_and_secrets(
                    crate::Kathy::AGENT_NAME,
                    &agent_home,
                    &remotes,
                    config,
                    &secrets,
                )
                .unwrap();
            assert_eq!(
                settings.agent.chat,
                ChatGenConfig::OrderedList {
                    messages: vec![
                        "Chat message 1".to_string(),
                        "Chat message 2".to_string(),
                        "Chat message 3".to_string(),
                    ]
                }
            );
            assert_eq!(settings.agent.interval, 999);
        })
        .await
    }

    async fn test_build_from_env_file(path: &str) {
        test_utils::run_test_with_env(path, || async move {
            let run_env = dotenv::var("RUN_ENV").unwrap();
            let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

            let settings = KathySettings::new().await.unwrap();

            let config = nomad_xyz_configuration::get_builtin(&run_env).unwrap();

            let remotes = get_remotes_from_env!(agent_home, config);
            let mut networks = remotes.clone();
            networks.insert(agent_home.clone());

            let secrets = AgentSecrets::from_env(&networks).unwrap();
            secrets
                .validate("kathy", &networks)
                .expect("!secrets validate");

            settings
                .base
                .validate_against_config_and_secrets(
                    crate::Kathy::AGENT_NAME,
                    &agent_home,
                    &remotes,
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
    async fn it_builds_settings_from_env() {
        test_build_from_env_file("../../fixtures/env.test").await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_builds_settings_from_partial_env() {
        test_build_from_env_file("../../fixtures/env.partial").await;
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_builds_settings_from_external_file() {
        test_utils::run_test_with_env("../../fixtures/env.external", || async move {
            std::env::set_var("CONFIG_PATH", "../../fixtures/external_config.json");
            let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

            let settings = KathySettings::new().await.unwrap();

            let config = nomad_xyz_configuration::NomadConfig::from_file(
                "../../fixtures/external_config.json",
            )
            .unwrap();

            let remotes = get_remotes_from_env!(agent_home, config);
            let mut networks = remotes.clone();
            networks.insert(agent_home.clone());

            let secrets = AgentSecrets::from_env(&networks).unwrap();
            secrets
                .validate("kathy", &networks)
                .expect("!secrets validate");

            settings
                .base
                .validate_against_config_and_secrets(
                    crate::Kathy::AGENT_NAME,
                    &agent_home,
                    &remotes,
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
