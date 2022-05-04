//! Configuration

use nomad_base::decl_settings;
use nomad_xyz_configuration::agent::relayer::RelayerConfig;

decl_settings!(Relayer, RelayerConfig,);

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

            let settings = RelayerSettings::new().unwrap();

            let config = nomad_xyz_configuration::get_builtin(&run_env).unwrap();
            let secrets = AgentSecrets::from_env(&config.networks).unwrap();

            settings
                .base
                .validate_against_config_and_secrets(
                    crate::Relayer::AGENT_NAME,
                    &agent_home,
                    config,
                    &secrets,
                )
                .unwrap();

            let interval = config.agent().get("ethereum").unwrap().relayer.interval;
            assert_eq!(settings.agent.interval, interval);
        })
        .await
    }
}
