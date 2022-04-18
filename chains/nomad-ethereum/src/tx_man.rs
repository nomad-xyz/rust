use color_eyre::{eyre::ensure, Result};
use ethers::{
    prelude::TransactionReceipt,
    providers::{EscalatingPending, Middleware, StreamExt},
    signers::Signer,
    types::transaction::eip2718::TypedTransaction,
};
use futures_util::stream::FuturesUnordered;
use nomad_core::db::TypedDB;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::{select, sync::mpsc::Receiver, task::JoinHandle, time::sleep};

/// A transaction lifecycle manager
#[derive(Debug)]
pub struct TxStorer<S, M> {
    next_nonce: AtomicU64,
    db: Arc<TypedDB>,
    signer: Arc<S>,
    provider: Arc<M>,
}

impl<S, M> TxStorer<S, M>
where
    S: Send + Sync + 'static + Signer,
    M: Send + Sync + 'static + Middleware,
{
    async fn store_receipt(&self, receipt: &TransactionReceipt) -> Result<()> {
        let tx_hash = receipt.transaction_hash;
        let tx = self.provider.get_transaction(tx_hash).await?;
        ensure!(
            tx.is_some(),
            "Tx is none, although receipt has confirmations. This is a weird RPC bug of some sort."
        );
        let tx = tx.expect("checked");

        // lots we want to log :/
        let block_hash = receipt
            .block_hash
            .expect("escalating only returns confirmed txns");
        let block_number = receipt
            .block_number
            .expect("escalating only returns confirmed txns");

        tracing::info!(
            tx_index = receipt.transaction_index.as_u64(),
            block = ?block_hash,
            block_height = block_number.as_u64(),
            tx_hash = ?tx_hash,
            nonce = tx.nonce.as_u64(),
            to = ?tx.to,
            data = %hex::encode(tx.input),
            "Storing receipt in DB",
        );

        // TODO: put receipt in db lol

        Ok(())
    }

    async fn handle_new(
        &self,
        mut tx: TypedTransaction,
    ) -> Result<EscalatingPending<'_, M::Provider>> {
        // logging setup
        let to_opt = tx.to();
        ensure!(to_opt.is_some(), "No to on transaction request");

        let next_nonce = self.next_nonce.fetch_add(1, Ordering::SeqCst);
        tx.set_nonce(next_nonce);

        tracing::info!(
            nonce = next_nonce,
            to = ?tx.to(),
            data = ?tx.data().map(hex::encode),
            "Storing tx request in DB"
        );

        self.db.store_encodable(
            "tx_storer_tx",
            next_nonce.to_be_bytes(),
            &serde_json::to_vec(&tx)?,
        )?;

        let escalating = self
            .provider
            .send_escalating(&tx, 5, Box::new(|original, index| original * (index + 1)))
            .await?;

        Ok(escalating)
    }

    async fn resume(&self) -> FuturesUnordered<EscalatingPending<'_, M::Provider>> {
        // TODO:
        // - poll chain for confirmed transaction count
        // - if that is less than the next_nonce
        //   - check DB for the intermediate txns
        //     - if they're not found,.... do what? TODO
        //   - rebroadcast
        //   - return the list of futures unordered of pending escalators
        // - if that is more than next_nonce..... do what? TODO

        Default::default()
    }

    /// Spawn the task
    pub fn spawn(self, mut inbound: Receiver<TypedTransaction>) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let mut escalators = self.resume().await;
            loop {
                sleep(std::time::Duration::from_millis(500)).await;

                // See: https://tokio.rs/tokio/tutorial/select
                select! {
                    // pattern = operation => handler
                    next_opt = inbound.recv() => {
                        // indicates that the sender has been dropped
                        if next_opt.is_none() {
                            tracing::info!("Inbound channel has closed. Shutting down TxManager task");
                            break;
                        }

                        let next = next_opt.expect("checked");
                        let esc = self.handle_new(next).await?;
                        escalators.push(esc);
                        continue;
                    }
                    // pattern = operation => handler
                    Some(result) = escalators.next() => {
                        let result = result?;
                        self.store_receipt(&result).await?;
                        continue;
                    }
                    // This path is reached if both of the above are disabled
                    // select paths are disabled when the operation returns a
                    // value that does not match the pattern
                    //
                    // i.e. if `self.inbound.recv()` returns `None`, it will NOT
                    // be polled again or re-run until `escalators.next()`
                    // resolves.
                    else => continue
                }
            }
            Ok(())
        })
    }
}
