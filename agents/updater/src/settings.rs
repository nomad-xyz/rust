//! Configuration
use nomad_base::decl_settings;
use nomad_xyz_configuration::agent::updater::UpdaterConfig;

decl_settings!(Updater, UpdaterConfig,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::{get_remotes_from_env, NomadAgent};
    use nomad_test::test_utils;
    use nomad_xyz_configuration::AgentSecrets;

    #[tokio::test]
    #[serial_test::serial]
    async fn it_builds_settings_from_env() {
        test_utils::run_test_with_env("../../fixtures/env.test", || async move {
            let run_env = dotenv::var("RUN_ENV").unwrap();
            let agent_home = dotenv::var("AGENT_HOME_NAME").unwrap();

            let settings = UpdaterSettings::new().unwrap();

            let config = nomad_xyz_configuration::get_builtin(&run_env).unwrap();

            let remotes = get_remotes_from_env!(agent_home, config);
            let mut networks = remotes.clone();
            networks.insert(agent_home.clone());

            let secrets = AgentSecrets::from_env(&networks).unwrap();

            settings
                .base
                .validate_against_config_and_secrets(
                    crate::Updater::AGENT_NAME,
                    &agent_home,
                    &remotes,
                    config,
                    &secrets,
                )
                .unwrap();

            let interval = config.agent().get("ethereum").unwrap().updater.interval;
            assert_eq!(settings.agent.interval, interval);
            assert_eq!(settings.base.attestation_signer, secrets.attestation_signer);
        })
        .await
    }
}
