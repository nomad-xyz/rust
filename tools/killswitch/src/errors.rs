use nomad_core::ChainCommunicationError;
use std::fmt::Display;

/// `Error` for KillSwitch
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No configuration found
    MissingConfig(String),
    /// Required network not found
    MissingNetwork(String),
    /// Required replica not found
    MissingReplicas(String),
    /// RPC config missing
    MissingRPC(String),
    /// Tx submitter config missing
    MissingTxSubmitter(String),
    /// Connection manager bad init
    ConnectionManagerInit(String),
    /// Signer failed to sign
    SignerFailed(String),
    /// `ChainCommunicationError` from tx submission
    ChainCommunicationError(#[from] ChainCommunicationError),
}

impl Display for Error {
    /// Display a detailed error message
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            MissingConfig(msg) => write!(f, "MissingConfig: {}", msg),
            MissingNetwork(msg) => write!(f, "MissingNetwork: {}", msg),
            MissingReplicas(msg) => write!(f, "MissingReplicas: {}", msg),
            _ => unimplemented!(),
        }
    }
}
