use color_eyre::Result;

#[derive(Debug)]
pub(crate) struct KillSwitchSettings {}

impl KillSwitchSettings {
    pub(crate) async fn new() -> Result<Self> {
        // Get config
        let config = if let Some(config_url) = std::env::var("CONFIG_URL").ok() {
            nomad_xyz_configuration::NomadConfig::fetch(&config_url)
                .await
                .expect(&format!("Unable to load config from {}", config_url))
        } else if let Some(config_path) = std::env::var("CONFIG_PATH").ok() {
            nomad_xyz_configuration::NomadConfig::from_file(&config_path)
                .expect(&format!("Unable to load config from {}", config_path))
        } else {
            color_eyre::eyre::bail!(
                "No configuration found. Set CONFIG_URL or CONFIG_PATH environment variable"
            )
        };
        config.validate()?;

        //

        return Ok(Self {});
    }
}
