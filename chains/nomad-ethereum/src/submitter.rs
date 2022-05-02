use crate::{SingleChainGelatoClient, ACCEPTABLE_STATES};
use color_eyre::Result;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use gelato_relay::RelayResponse;
use nomad_core::{ChainCommunicationError, TxOutcome};
use std::{str::FromStr, sync::Arc};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// Component responsible for submitting transactions to the chain. Can
/// sign/submit locally or use a transaction relay service.
#[derive(Debug, Clone)]
pub enum SubmitterClient<M> {
    /// Sign/submit txs locally
    Local(Arc<M>),
    /// Pass meta txs to Gelato relay service
    Gelato(SingleChainGelatoClient<M>),
}

impl<M> From<Arc<M>> for SubmitterClient<M> {
    fn from(client: Arc<M>) -> Self {
        Self::Local(client)
    }
}

impl<M> From<SingleChainGelatoClient<M>> for SubmitterClient<M> {
    fn from(client: SingleChainGelatoClient<M>) -> Self {
        Self::Gelato(client)
    }
}

/// Chain submitter
#[derive(Debug)]
pub struct TxSubmitter<M> {
    /// Tx submitter client
    pub client: SubmitterClient<M>,
}

impl<M> TxSubmitter<M>
where
    M: Middleware + 'static,
{
    /// Create new TxSubmitter from submitter
    pub fn new(client: SubmitterClient<M>) -> Self {
        Self { client }
    }

    /// Submit transaction to chain
    pub async fn submit(
        &self,
        domain: u32,
        contract_address: Address,
        tx: impl Into<TypedTransaction>,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        let tx: TypedTransaction = tx.into();

        match &self.client {
            SubmitterClient::Local(client) => {
                let dispatched = client
                    .send_transaction(tx, None)
                    .await
                    .map_err(|e| ChainCommunicationError::TxSubmissionError(e.into()))?;
                let tx_hash: ethers::core::types::H256 = *dispatched;
                info!("dispatched transaction with tx_hash {:?}", tx_hash);

                let result = dispatched
                    .await?
                    .ok_or_else(|| ChainCommunicationError::DroppedError(tx_hash))?;

                info!(
                    "confirmed transaction with tx_hash {:?}",
                    result.transaction_hash
                );

                let outcome = result.try_into()?;
                Ok(outcome)
            }
            SubmitterClient::Gelato(client) => {
                let tx_data = tx.data().expect("!tx data");
                let data = format!("{:x}", tx_data);
                let address = format!("{:x}", contract_address);

                info!(
                    domain = domain,
                    contract_address = ?address,
                    "Dispatching tx to Gelato relay."
                );

                let gas_limit = 100_000; // TODO: clear up with Gelato
                let RelayResponse { task_id } = client
                    .send_relay_transaction(&address, &data, gas_limit)
                    .await
                    .map_err(|e| ChainCommunicationError::TxSubmissionError(e.into()))?;
                info!(task_id = ?task_id, "Submitted tx to Gelato relay. Polling task for completion...");

                loop {
                    let status = client
                        .client()
                        .get_task_status(&task_id)
                        .await
                        .map_err(|e| ChainCommunicationError::TxSubmissionError(e.into()))?
                        .expect("!task status");

                    if !ACCEPTABLE_STATES.contains(&status.task_state) {
                        return Err(ChainCommunicationError::TxSubmissionError(
                            format!("Gelato task failed: {:?}", status).into(),
                        )
                        .into());
                    }

                    if let Some(execution) = &status.execution {
                        info!(
                            chain = ?status.chain,
                            task_id = ?status.task_id,
                            execution = ?execution,
                            "Gelato relay executed tx."
                        );

                        let tx_hash = &execution.transaction_hash;
                        let txid = H256::from_str(tx_hash)
                            .unwrap_or_else(|_| panic!("Malformed tx hash from Gelato"));

                        return Ok(TxOutcome { txid });
                    }

                    debug!(task_id = ?task_id, "Polling Gelato task.");
                    sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }
}
