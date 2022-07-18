use crate::NomadDB;
use color_eyre::Result;
use nomad_core::{ChainCommunicationError, NomadTxStatus, PersistedTransaction, TxOutcome};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        oneshot, Mutex,
    },
    task::JoinHandle,
    time::sleep,
};

/// Tokio oneshot channel for returning tx results to sender handle
pub type TxResultChannel = oneshot::Sender<Result<TxOutcome, ChainCommunicationError>>;

const MAX_TRANSACTIONS_PER_DB_CALL: usize = 10;
const SEND_TASK_LOOP_SLEEP_MS: u64 = 100;

/// TxSender that receives outgoing transactions, polls for status and forwards to
/// concrete contract impl to translate and submit transactions
#[derive(Debug)]
pub struct TxSender {
    db: NomadDB,
    in_sender: UnboundedSender<(PersistedTransaction, TxResultChannel)>,
    in_receiver: Option<UnboundedReceiver<(PersistedTransaction, TxResultChannel)>>,
    out_sender: UnboundedSender<PersistedTransaction>,
    out_receiver: Option<UnboundedReceiver<PersistedTransaction>>,
    outcome_senders: Arc<Mutex<HashMap<u64, TxResultChannel>>>,
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
            outcome_senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Take out_receiver for the run loop
    pub fn take_out_receiver(&mut self) -> Option<UnboundedReceiver<PersistedTransaction>> {
        self.out_receiver.take()
    }

    /// Clone in_sender for external use
    pub fn in_sender(&self) -> UnboundedSender<(PersistedTransaction, TxResultChannel)> {
        self.in_sender.clone()
    }

    /// Spawn run loop task
    pub fn send_task(&mut self) -> Option<JoinHandle<Result<()>>> {
        let mut in_receiver = self.in_receiver.take().unwrap();
        let out_sender = self.out_sender.clone();
        let db = self.db.clone();
        let outcome_senders = self.outcome_senders.clone();
        Some(tokio::spawn(async move {
            loop {
                // Accept and store new txs
                if let Ok((tx, outcome_sender)) = in_receiver.try_recv() {
                    assert_eq!(tx.status, NomadTxStatus::NotSent);
                    let counter = db.store_persisted_transaction(tx)?;
                    outcome_senders.lock().await.insert(counter, outcome_sender);
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
                            let outcome_sender = outcome_senders
                                .lock()
                                .await
                                .remove(&tx.counter)
                                .expect("!outcome_sender (should never happen)");
                            outcome_sender
                                .send(Ok(TxOutcome::Dummy))
                                .expect("!oneshot receiver (should never happen)");
                            tx.status = NomadTxStatus::Finalized;
                            db.update_persisted_transaction(&tx)?;
                        }
                        _ => unreachable!(),
                    }
                }
                sleep(Duration::from_millis(SEND_TASK_LOOP_SLEEP_MS)).await;
            }
        }))
    }
}

impl std::fmt::Display for TxSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
