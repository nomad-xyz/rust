use color_eyre::{eyre, Result};
use nomad_core::Signers;
use nomad_xyz_configuration::{AgentSecrets, NomadConfig};
use std::{collections::HashMap, env};

#[derive(Debug)]
pub(crate) struct KillSwitchSettings {
    secrets: HashMap<String, AgentSecrets>,
    signers: HashMap<String, Signers>,
}

impl KillSwitchSettings {
    pub(crate) async fn new() -> Result<Self> {
        // Get config
        let config = if let Some(config_url) = env::var("CONFIG_URL").ok() {
            NomadConfig::fetch(&config_url)
                .await
                .expect(&format!("Unable to load config from {}", config_url))
        } else if let Some(config_path) = env::var("CONFIG_PATH").ok() {
            NomadConfig::from_file(&config_path)
                .expect(&format!("Unable to load config from {}", config_path))
        } else {
            eyre::bail!(
                "No configuration found. Set CONFIG_URL or CONFIG_PATH environment variable"
            )
        };
        config.validate()?;

        //

        return Ok(Self {
            secrets: HashMap::new(),
            signers: HashMap::new(),
        });
    }
}
