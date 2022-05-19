//! Configuration

use nomad_base::decl_settings;
use nomad_xyz_configuration::agent::kathy::KathyConfig;

decl_settings!(Kathy, KathyConfig,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::{get_remotes_from_env, NomadAgent};
    use nomad_test::test_utils;
    use nomad_xyz_configuration::{
        agent::SignerConf, ethereum::Connection, AgentSecrets, ChainConf,
    };

    #[tokio::test]
    #[serial_test::serial]
    async fn it_builds_settings_from_env_mixed() {
        test_utils::run_test_with_env("../../fixtures/env.test-signer-mixed", || async move {
            let run_env = dotenv::var("RUN_ENV").unwrap();
            let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

            let settings = KathySettings::new().unwrap();

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
                *settings.base.signers.get("moonbeam").unwrap(),
                SignerConf::Aws {
                    id: "moonbeam_id".into(),
                    region: "moonbeam_region".into(),
                }
            );
            assert_eq!(
                *settings.base.signers.get("ethereum").unwrap(),
                SignerConf::HexKey(
                    "0x1111111111111111111111111111111111111111111111111111111111111111"
                        .parse()
                        .unwrap()
                )
            );
            assert_eq!(
                *settings.base.signers.get("evmos").unwrap(),
                SignerConf::Aws {
                    id: "default_id".into(),
                    region: "default_region".into(),
                }
            );
            assert_eq!(
                settings.base.home.chain,
                ChainConf::Ethereum(Connection::Http(
                    "https://main-light.eth.linkpool.io/".into()
                ))
            );
            assert_eq!(
                settings.base.replicas.get("moonbeam").unwrap().chain,
                ChainConf::Ethereum(Connection::Http("https://rpc.api.moonbeam.network".into()))
            );
            assert_eq!(
                settings.base.replicas.get("evmos").unwrap().chain,
                ChainConf::Ethereum(Connection::Http("https://eth.bd.evmos.org:8545".into()))
            );
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_builds_settings_from_env_default() {
        test_utils::run_test_with_env("../../fixtures/env.test-signer-default", || async move {
            let run_env = dotenv::var("RUN_ENV").unwrap();
            let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

            let settings = KathySettings::new().unwrap();

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

            let default_config = SignerConf::Aws {
                id: "default_id".into(),
                region: "default_region".into(),
            };
            for (_, config) in &settings.base.signers {
                assert_eq!(*config, default_config);
            }
            assert!(matches!(
                settings.base.home.chain,
                ChainConf::Ethereum { .. }
            ));
            for (_, config) in &settings.base.replicas {
                assert!(matches!(config.chain, ChainConf::Ethereum { .. }));
            }
        })
        .await
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn it_builds_settings_from_env() {
        test_utils::run_test_with_env("../../fixtures/env.test", || async move {
            let run_env = dotenv::var("RUN_ENV").unwrap();
            let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

            let settings = KathySettings::new().unwrap();

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
            let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

            let settings = KathySettings::new().unwrap();

            let config = nomad_xyz_configuration::NomadConfig::from_file(
                "../../fixtures/external_config.json",
            )
            .unwrap();

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

    #[tokio::test]
    #[serial_test::serial]
    async fn it_builds_settings_from_partial_env() {
        test_utils::run_test_with_env("../../fixtures/env.partial", || async move {
            let run_env = dotenv::var("RUN_ENV").unwrap();
            let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

            let settings = KathySettings::new().unwrap();

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

            let agent_config = &config.agent().get("ethereum").unwrap().kathy;
            assert_eq!(settings.agent.interval, agent_config.interval);
            assert_eq!(settings.agent.chat, agent_config.chat);
        })
        .await
    }
}
