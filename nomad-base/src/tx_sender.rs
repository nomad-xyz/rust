use crate::NomadDB;
use color_eyre::Result;
use nomad_core::PersistedTransaction;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

/// TxSender that receives outgoing transactions, polls for status and forwards to
/// concrete contract impl to translate and submit transactions
#[derive(Debug)]
pub struct TxSender {
    db: NomadDB,
    in_sender: UnboundedSender<PersistedTransaction>,
    in_receiver: Option<UnboundedReceiver<PersistedTransaction>>,
    out_sender: UnboundedSender<PersistedTransaction>,
    out_receiver: Option<UnboundedReceiver<PersistedTransaction>>,
}

impl TxSender {
    /// Create a new TxPoller with a DB and an UnboundedReceiver
    pub fn new(db: NomadDB) -> Self {
        let (in_sender, in_receiver) = unbounded_channel();
        let (out_sender, out_receiver) = unbounded_channel();
        Self {
            db,
            in_sender,
            in_receiver: Some(in_receiver),
            out_sender,
            out_receiver: Some(out_receiver),
        }
    }

    /// Take out_receiver for the run loop
    pub fn take_out_receiver(&mut self) -> Option<UnboundedReceiver<PersistedTransaction>> {
        self.out_receiver.take()
    }

    /// Clone in_sender for external use
    pub fn in_sender(&self) -> UnboundedSender<PersistedTransaction> {
        self.in_sender.clone()
    }

    /// Spawn run loop task
    pub fn send_task(&mut self) -> Option<JoinHandle<Result<()>>> {
        let mut in_receiver = self.in_receiver.take().unwrap();
        Some(tokio::spawn(async move {
            loop {
                if let Ok(_tx) = in_receiver.try_recv() {
                    unimplemented!()
                }
            }
        }))
    }
}

impl std::fmt::Display for TxSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
