//! Configuration

use nomad_base::decl_settings;
use nomad_xyz_configuration::agent::watcher::WatcherConfig;

decl_settings!(Watcher, WatcherConfig,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::{AgentSecrets, NomadAgent};
    use nomad_xyz_configuration::contracts::CoreContracts;

    const RUN_ENV: &str = "test";
    const AGENT_HOME: &str = "ethereum";
    const SECRETS_PATH: &str = "../../fixtures/secrets.json";

    #[test]
    fn it_builds_settings_from_config_and_secrets() {
        std::env::set_var("RUN_ENV", RUN_ENV);
        std::env::set_var("AGENT_HOME", AGENT_HOME);
        std::env::set_var("SECRETS_PATH", SECRETS_PATH);

        let settings = WatcherSettings::new().unwrap();

        let config = nomad_xyz_configuration::get_builtin(RUN_ENV).unwrap();
        let secrets = AgentSecrets::from_file(SECRETS_PATH).unwrap();

        settings
            .base
            .validate_against_config_and_secrets(
                crate::Watcher::AGENT_NAME,
                AGENT_HOME,
                config,
                &secrets,
            )
            .unwrap();

        let agent_config = &config.agent().get("ethereum").unwrap().watcher;
        assert_eq!(settings.agent.interval, agent_config.interval);
        assert_eq!(settings.base.attestation_signer, secrets.attestation_signer);

        let home_connections = &config
            .protocol()
            .networks
            .get(AGENT_HOME)
            .expect("!networks")
            .connections;

        for remote_network in home_connections {
            let manager_setup = settings
                .as_ref()
                .managers
                .as_ref()
                .unwrap()
                .get(remote_network)
                .unwrap();

            let config_manager_domain = config
                .protocol()
                .get_network(remote_network.to_owned().into())
                .unwrap();

            assert_eq!(manager_setup.name, config_manager_domain.name);
            assert_eq!(manager_setup.domain, config_manager_domain.domain);
            assert_eq!(
                manager_setup.page_settings.page_size,
                config_manager_domain.specs.index_page_size
            );
            assert_eq!(
                manager_setup.finality,
                config_manager_domain.specs.finalization_blocks
            );

            let config_manager_core = config.core().get(remote_network).unwrap();
            match config_manager_core {
                CoreContracts::Evm(core) => {
                    assert_eq!(manager_setup.address, core.x_app_connection_manager,);
                    assert_eq!(manager_setup.page_settings.from, core.deploy_height);
                }
            }

            let manager_chain_conf = secrets.rpcs.get(remote_network).unwrap();
            assert_eq!(&manager_setup.chain, manager_chain_conf);
        }
    }
}
