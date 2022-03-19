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
