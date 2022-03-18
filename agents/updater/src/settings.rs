//! Configuration
use nomad_base::{decl_settings, AgentSecrets, AgentSettingsBlock};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdaterSettingsBlock {
    pub interval: u64,
    pub attestation_signer: nomad_base::SignerConf,
}

impl AgentSettingsBlock for UpdaterSettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        secrets: &AgentSecrets,
    ) -> Self {
        let interval = config.agent().get(home_network).unwrap().updater.interval;
        let attestation_signer = secrets.attestation_signer.clone();
        Self {
            interval,
            attestation_signer,
        }
    }
}
decl_settings!(Updater, UpdaterSettingsBlock,);
