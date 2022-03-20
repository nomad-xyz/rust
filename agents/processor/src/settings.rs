//! Configuration
use ethers::prelude::H256;
use nomad_base::{decl_settings, AgentSecrets, AgentSettingsBlock};
use nomad_xyz_configuration::agent::processor::S3Config;
use std::collections::HashSet;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProcessorSettingsBlock {
    /// The polling interval (in seconds)
    pub interval: u64,
    /// An allow list of message senders
    pub allowed: Option<HashSet<H256>>,
    /// A deny list of message senders
    pub denied: Option<HashSet<H256>>,
    /// Only index transactions if this key is set
    pub index_only: bool,
    /// An amazon aws s3 bucket to push proofs to
    pub s3: Option<S3Config>,
}

impl AgentSettingsBlock for ProcessorSettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        _secrets: &AgentSecrets,
    ) -> Self {
        let config = &config.agent().get(home_network).unwrap().processor;
        Self {
            interval: config.interval,
            allowed: config.allowed.clone(),
            denied: config.denied.clone(),
            index_only: config.index_only,
            s3: config.s3.clone(),
        }
    }
}
decl_settings!(Processor, ProcessorSettingsBlock,);

#[cfg(test)]
mod test {
    use super::*;
    use nomad_base::NomadAgent;

    const RUN_ENV: &str = "test";
    const AGENT_HOME: &str = "ethereum";
    const SECRETS_PATH: &str = "../../fixtures/secrets.json";

    #[test]
    fn it_builds_settings_from_config_and_secrets() {
        std::env::set_var("RUN_ENV", RUN_ENV);
        std::env::set_var("AGENT_HOME", AGENT_HOME);
        std::env::set_var("SECRETS_PATH", SECRETS_PATH);

        let settings = ProcessorSettings::new().unwrap();

        let config = nomad_xyz_configuration::get_builtin(RUN_ENV).unwrap();
        let secrets = AgentSecrets::from_file(SECRETS_PATH.into()).unwrap();

        settings
            .base
            .validate_against_config_and_secrets(
                crate::Processor::AGENT_NAME,
                AGENT_HOME,
                config,
                &secrets,
            )
            .unwrap();

        let agent_config = &config.agent().get("ethereum").unwrap().processor;
        assert_eq!(settings.agent.interval, agent_config.interval);
        assert_eq!(settings.agent.allowed, agent_config.allowed);
        assert_eq!(settings.agent.denied, agent_config.denied);
        assert_eq!(settings.agent.index_only, agent_config.index_only);
        assert_eq!(settings.agent.s3, agent_config.s3);
    }
}
