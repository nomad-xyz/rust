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

/// Takes a struct member of type Option<Result<V, Error>>
/// and returns `true` if `is_some() && is_err() == true`
macro_rules! is_error {
    ($member:expr) => {{
        $member
            .as_ref()
            .map(|result| result.is_err())
            .unwrap_or(false)
    }};
}
pub(crate) use is_error;

/// Takes a struct member of type Option<Result<V, Error>>
/// and returns Some<Error> if `is_some() && is_err() == true`
macro_rules! take_error {
    ($member:expr) => {{
        if is_error!($member) {
            ::core::option::Option::Some($member.take().unwrap().unwrap_err())
        } else {
            ::core::option::Option::None
        }
    }};
}
pub(crate) use take_error;
