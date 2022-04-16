use color_eyre::{eyre::ensure, Result};
use ethers::{
    prelude::{NameOrAddress, TransactionReceipt},
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

fn log_tx(tx: &TypedTransaction) -> (String, String) {
    let to = match tx.to().expect("checked") {
        NameOrAddress::Name(name) => name.to_owned(),
        NameOrAddress::Address(address) => hex::encode(address),
    };
    let data = tx
        .data()
        .map(hex::encode)
        .unwrap_or_else(|| "0x".to_owned());

    (to, data)
}

/// A transaction lifecycle manager
#[derive(Debug)]
pub struct TxStorer<S, M> {
    inbound: Receiver<TypedTransaction>,
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
        let to = tx.to.map(hex::encode).unwrap_or_else(|| "0x".to_owned());
        let data = hex::encode(&tx.input);

        tracing::info!(
            tx_index = receipt.transaction_index.as_u64(),
            block = hex::encode(&block_hash).as_str(),
            block_height = block_number.as_u64(),
            tx_hash = hex::encode(&tx_hash).as_str(),
            nonce = tx.nonce.as_u64(),
            to = to.as_str(),
            data = data.as_str(),
            "Storing receipt in DB",
        );

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
        let (to, data) = log_tx(&tx);

        tracing::info!(
            nonce = next_nonce,
            to = to.as_str(),
            data = data.as_str(),
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

    /// Spawn the task
    pub async fn spawn(mut self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            loop {
                sleep(std::time::Duration::from_millis(500)).await;

                let mut escalators = FuturesUnordered::new();

                select! {
                    Some(next) = self.inbound.recv() => {
                        let esc = self.handle_new(next).await?;
                        escalators.push(esc);
                    }
                    Some(result) = escalators.next() => {
                        let result = result?;
                        self.store_receipt(&result).await?;
                    }
                    else => continue
                }
            }
        })
    }
}
