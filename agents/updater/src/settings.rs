//! Configuration
use nomad_base::decl_settings;
use nomad_xyz_configuration::agent::updater::UpdaterConfig;

decl_settings!(Updater, UpdaterConfig,);

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

        let settings = UpdaterSettings::new().unwrap();

        let config = nomad_xyz_configuration::get_builtin(RUN_ENV).unwrap();
        let secrets = AgentSecrets::from_file(SECRETS_PATH).unwrap();

        settings
            .base
            .validate_against_config_and_secrets(
                crate::Updater::AGENT_NAME,
                AGENT_HOME,
                config,
                &secrets,
            )
            .unwrap();

        let interval = config.agent().get("ethereum").unwrap().updater.interval;
        assert_eq!(settings.agent.interval, interval);
        assert_eq!(settings.base.attestation_signer, secrets.attestation_signer);
    }
}
