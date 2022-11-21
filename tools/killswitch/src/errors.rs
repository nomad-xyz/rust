use nomad_base::ChainCommunicationError;
use nomad_ethereum::EthereumSignersError;
use rusoto_core::credential::CredentialsError;

/// `Error` for KillSwitch
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Cannot find local AWS credentials
    #[error("CredentialsError: Cannot find AWS credentials: {0}")]
    CredentialsError(#[source] CredentialsError),
    /// No configuration env var
    #[error(
        "NoConfigVar: No configuration found. Set CONFIG_URL or CONFIG_PATH environment variable"
    )]
    NoConfigVar,
    /// Bad configuration env var
    #[error("BadConfigVar: Unable to load config from: {0}")]
    BadConfigVar(String),
    /// No killable networks found
    #[error("NoNetworks: No available networks in config to kill")]
    NoNetworks,
    /// RPC config missing
    #[error("MissingRPC: No rpc config found for: {0}")]
    MissingRPC(String),
    /// Tx submitter config missing
    #[error("MissingTxSubmitterConf: No transaction submitter config found for: {0}")]
    MissingTxSubmitterConf(String),
    /// Attestation signer config missing
    #[error("MissingAttestationSignerConf: No attestation signer config found for: {0}")]
    MissingAttestationSignerConf(String),
    /// Home bad init
    #[error("HomeInit: Home init failed: {0}")]
    HomeInit(String),
    /// Connection manager bad init
    #[error("ConnectionManagerInit: Connection manager init failed: {0}")]
    ConnectionManagerInit(String),
    /// Attestation signer bad init
    #[error("AttestationSignerInit: Attestation signer init failed: {0}")]
    AttestationSignerInit(String),
    /// Can't get updater address
    #[error("UpdaterAddress: Error getting updater address: {0}")]
    UpdaterAddress(#[source] ChainCommunicationError),
    /// Attestation signer failed to sign
    #[error("AttestationSignerFailed: Attestation signer failed: {0}")]
    AttestationSignerFailed(#[source] EthereumSignersError),
    /// Unenrollment failure
    #[error("UnenrollmentFailed: Unenrollment failed: {0}")]
    UnenrollmentFailed(#[source] ChainCommunicationError),
}
