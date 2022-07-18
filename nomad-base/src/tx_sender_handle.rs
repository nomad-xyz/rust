use color_eyre::Result;
use nomad_core::{ChainCommunicationError, PersistedTransaction, TxOutcome};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

/// Transaction manager for handling PersistentTransaction
#[derive(Debug, Clone)]
pub struct TxSenderHandle {
    sender: UnboundedSender<(
        PersistedTransaction,
        oneshot::Sender<Result<TxOutcome, ChainCommunicationError>>,
    )>,
}

impl TxSenderHandle {
    /// Create a new TxSenderHandle with a sender
    pub fn new(
        sender: UnboundedSender<(
            PersistedTransaction,
            oneshot::Sender<Result<TxOutcome, ChainCommunicationError>>,
        )>,
    ) -> Self {
        Self { sender }
    }

    /// Send a PersistedTransaction ...
    pub fn send(&self, _tx: PersistedTransaction) -> Result<TxOutcome, ChainCommunicationError> {
        let _sender = &self.sender;
        unimplemented!()
    }

    /// Send a PersistedTransaction ...
    pub fn send_blocking(
        &self,
        _tx: PersistedTransaction,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        unimplemented!()
    }
}

impl std::fmt::Display for TxSenderHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
