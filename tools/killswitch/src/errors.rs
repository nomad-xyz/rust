use nomad_core::{ChainCommunicationError, SignersError};

/// `Error` for KillSwitch
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No configuration env var
    #[error("No configuration found. Set CONFIG_URL or CONFIG_PATH environment variable")]
    NoConfigVar,
    /// Bad configuration env var
    #[error("Unable to load config from {0}")]
    BadConfigVar(String),
    /// No killable networks found
    #[error("No available networks in config to kill")]
    NoNetworks,
    /// RPC config missing
    #[error("No rpc config found for network: {0}")]
    MissingRPC(String),
    /// Tx submitter config missing
    #[error("No transaction submitter config found for {0}")]
    MissingTxSubmitterConf(String),
    /// Attestation signer config missing
    #[error("No attestation signer config found for {0}")]
    MissingAttestationSignerConf(String),
    /// Home bad init
    #[error("Home init failed: {0}")]
    HomeInit(String),
    /// Connection manager bad init
    #[error("Connection manager init failed: {0}")]
    ConnectionManagerInit(String),
    /// Attestation signer bad init
    #[error("Attestation signer init failed: {0}")]
    AttestationSignerInit(String),
    /// Can't get updater address
    #[error("Error getting updater address: {0}")]
    UpdaterAddress(#[source] ChainCommunicationError),
    /// Attestation signer failed to sign
    #[error("Attestation signer failed: {0}")]
    AttestationSignerFailed(#[source] SignersError),
    /// Unenrollment failure
    #[error("Unenrollment failed: {0}")]
    UnenrollmentFailed(#[source] ChainCommunicationError),
}
