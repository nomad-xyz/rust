//! Configuration
use ethers::prelude::H256;
use serde::Deserialize;
use std::collections::HashSet;

use nomad_base::{decl_settings, AgentSecrets, AgentSettingsBlock};

#[derive(Debug, Deserialize, Clone)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProcessorSettingsBlock {
    /// The polling interval (in seconds)
    pub interval: u64,
    /// An allow list of message senders
    pub allowed: Option<HashSet<H256>>,
    /// A deny list of message senders
    pub denied: Option<HashSet<H256>>,
    /// Only index transactions if this key is set
    pub indexon: Option<String>,
    /// An amazon aws s3 bucket to push proofs to
    pub s3: Option<S3Config>,
}

// TODO: add processor settings block to nomad-types
impl AgentSettingsBlock for ProcessorSettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        _secrets: &AgentSecrets,
    ) -> Self {
        let interval = config.agent().get(home_network).unwrap().processor.interval;
        Self {
            interval,
            allowed: Default::default(),
            denied: Default::default(),
            indexon: Default::default(),
            s3: Some(S3Config {
                bucket: Default::default(),
                region: Default::default(),
            }),
        }
    }
}
decl_settings!(Processor, ProcessorSettingsBlock,);
