use color_eyre::Result;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use gelato_relay::{RelayResponse, SingleChainGelatoClient};
use std::sync::Arc;
use tracing::info;

/// Component responsible for submitting transactions to the chain. Can
/// sign/submit locally or use a transaction relay service.
#[derive(Debug, Clone)]
pub enum Submitter<M> {
    /// Sign/submit txs locally
    Local(Arc<M>),
    /// Pass meta txs to Gelato relay service
    Gelato(SingleChainGelatoClient<M>),
}

impl<M> From<Arc<M>> for Submitter<M> {
    fn from(client: Arc<M>) -> Self {
        Self::Local(client)
    }
}

impl<M> From<SingleChainGelatoClient<M>> for Submitter<M> {
    fn from(client: SingleChainGelatoClient<M>) -> Self {
        Self::Gelato(client)
    }
}

/// Receives meta txs and submits them to chain
#[derive(Debug)]
pub struct ChainSubmitter<M> {
    /// Tx submitter
    pub submitter: Submitter<M>,
}

impl<M: Middleware + 'static> ChainSubmitter<M> {
    /// Submit transaction to chain
    pub async fn submit(
        &self,
        domain: u32,
        contract_address: Address,
        tx: impl Into<TypedTransaction>,
    ) -> Result<()> {
        let tx: TypedTransaction = tx.into();

        match &self.submitter {
            Submitter::Local(client) => {
                let dispatched = client.send_transaction(tx, None).await?;
                let tx_hash: ethers::core::types::H256 = *dispatched;
                info!("dispatched transaction with tx_hash {:?}", tx_hash);

                let result = dispatched
                    .await?
                    .ok_or_else(|| nomad_core::ChainCommunicationError::DroppedError(tx_hash))?;

                info!(
                    "confirmed transaction with tx_hash {:?}",
                    result.transaction_hash
                );
            }
            Submitter::Gelato(client) => {
                let tx_data = tx.data().expect("!tx data");
                let data = format!("{:x}", tx_data);
                let address = format!("{:x}", contract_address);

                info!(
                    domain = domain,
                    contract_address = ?address,
                    "Dispatching tx to Gelato relay."
                );

                let RelayResponse { task_id } =
                    client.send_relay_transaction(&address, &data).await?;
                info!(task_id = ?task_id, "Submitted tx to Gelato relay.");

                loop {
                    let status = client
                        .client()
                        .get_task_status(&task_id)
                        .await?
                        .expect("!task status");

                    if let Some(execution) = &status.execution {
                        info!(
                            chain = ?status.chain,
                            task_id = ?status.task_id,
                            execution = ?execution,
                            "Gelato relay executed tx."
                        );

                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
