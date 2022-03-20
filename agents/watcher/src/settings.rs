//! Configuration

use nomad_base::{
    decl_settings, AgentSecrets, AgentSettingsBlock, ChainSetup, ChainSetupType, SignerConf,
};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WatcherSettingsBlock {
    pub interval: u64,
    pub managers: HashMap<String, ChainSetup>,
    pub attestation_signer: SignerConf,
}

impl AgentSettingsBlock for WatcherSettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        secrets: &AgentSecrets,
    ) -> Self {
        let interval = config.agent().get(home_network).unwrap().watcher.interval;

        let home_connections = &config
            .protocol()
            .networks
            .get(home_network)
            .expect("!networks")
            .connections;

        // Connection manager has one xapp for every home connection
        let managers: HashMap<String, ChainSetup> = home_connections
            .iter()
            .map(|remote_network| {
                (
                    remote_network.to_owned(),
                    ChainSetup::from_config_and_secrets(
                        ChainSetupType::ConnectionManager { remote_network },
                        config,
                        secrets,
                    ),
                )
            })
            .collect();

        let attestation_signer = secrets.attestation_signer.clone();

        Self {
            interval,
            managers,
            attestation_signer,
        }
    }
}
decl_settings!(Watcher, WatcherSettingsBlock,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::NomadAgent;
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
        let secrets = AgentSecrets::from_file(SECRETS_PATH.into()).unwrap();

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
        assert_eq!(
            settings.agent.attestation_signer,
            secrets.attestation_signer
        );

        let home_connections = &config
            .protocol()
            .networks
            .get(AGENT_HOME)
            .expect("!networks")
            .connections;

        for remote_network in home_connections {
            let manager_setup = settings.agent.managers.get(remote_network).unwrap();

            let config_manager_domain = config
                .protocol()
                .get_network(remote_network.to_owned().into())
                .unwrap();

            assert_eq!(manager_setup.name, config_manager_domain.name);
            assert_eq!(manager_setup.domain as u64, config_manager_domain.domain);
            assert_eq!(
                manager_setup.page_settings.page_size as u64,
                config_manager_domain.specs.index_page_size
            );
            assert_eq!(
                manager_setup.finality as u64,
                config_manager_domain.specs.finalization_blocks
            );

            let config_manager_core = config.core().get(remote_network).unwrap();
            match config_manager_core {
                CoreContracts::Evm(core) => {
                    assert_eq!(manager_setup.address, core.x_app_connection_manager,);
                    assert_eq!(manager_setup.page_settings.from as u64, core.deploy_height);
                }
            }

            let manager_chain_conf = secrets.rpcs.get(remote_network).unwrap();
            assert_eq!(&manager_setup.chain, manager_chain_conf);
        }
    }
}
