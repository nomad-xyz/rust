use crate::NomadDB;
use color_eyre::Result;
use nomad_core::{NomadTxStatus, PersistedTransaction};
use std::time::Duration;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

const MAX_TRANSACTIONS_PER_DB_CALL: usize = 10;
const SEND_TASK_LOOP_SLEEP_MS: u64 = 100;

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
        let out_sender = self.out_sender.clone();
        let db = self.db.clone();
        Some(tokio::spawn(async move {
            loop {
                // Accept and store new txs
                if let Ok(mut tx) = in_receiver.try_recv() {
                    assert_eq!(tx.status, NomadTxStatus::NotSent);
                    db.store_persisted_transaction(tx)?;
                }
                for mut tx in db.persisted_transactions(
                    vec![NomadTxStatus::NotSent, NomadTxStatus::Successful],
                    MAX_TRANSACTIONS_PER_DB_CALL,
                ) {
                    match tx.status {
                        // Send out new txs, mark pending
                        NomadTxStatus::NotSent => {
                            out_sender.send(tx.clone())?;
                            tx.status = NomadTxStatus::Pending;
                            db.update_persisted_transaction(&tx)?;
                        }
                        // Finalize successful txs
                        NomadTxStatus::Successful => {
                            tx.status = NomadTxStatus::Finalized;
                            db.update_persisted_transaction(&tx)?;
                        }
                        _ => unreachable!(),
                    }
                }
                tokio::time::sleep(Duration::from_millis(SEND_TASK_LOOP_SLEEP_MS)).await;
            }
        }))
    }
}

impl std::fmt::Display for TxSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
