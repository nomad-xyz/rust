use color_eyre::Result;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use gelato_relay::{RelayResponse, SingleChainGelatoClient};
use nomad_core::Signers;
use nomad_xyz_configuration::ethereum::Connection;
use std::sync::Arc;
use tracing::info;

/// Configuration for a ethers signing provider
#[derive(Debug, Clone)]
pub struct SigningProviderConfig {
    /// Signer configuration
    pub signer: Signers,
    /// Connection configuration
    pub connection: Connection,
}

impl SigningProviderConfig {
    /// Instantiate new signing provider config
    pub fn new(signer: Signers, connection: Connection) -> Self {
        Self { signer, connection }
    }
}

/// Unsigned transaction and metadata
#[derive(Debug, Clone)]
pub struct MetaTx {
    domain: u32,
    contract_address: Address,
    tx: TypedTransaction,
}

/// Component responsible for submitting transactions to the chain. Can
/// sign/submit locally or use a transaction relay service.
#[derive(Debug, Clone)]
pub enum Submitter<M: Middleware + 'static> {
    /// Sign/submit txs locally
    Local(Arc<M>),
    /// Pass meta txs to Gelato relay service
    Gelato(SingleChainGelatoClient<M>),
}

/// Receives meta txs and submits them to chain
#[derive(Debug)]
pub struct ChainSubmitter<M: Middleware + 'static> {
    /// Tx submitter
    pub submitter: Submitter<M>,
    // /// Meta tx receiver
    // pub rx: mpsc::Receiver<MetaTx>,
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

    // /// Spawn ChainSubmitter task. Receives meta txs and submits in loop.
    // #[instrument]
    // pub async fn spawn(mut self) -> JoinHandle<Result<()>> {
    //     tokio::spawn(async move {
    //         loop {
    //             let tx = self.rx.recv().await;

    //             if tx.is_none() {
    //                 bail!("Eth ChainSubmitter channel closed.")
    //             }

    //             let MetaTx {
    //                 domain,
    //                 contract_address,
    //                 tx,
    //             } = tx.unwrap();

    //             self.submit(domain, contract_address, tx).await?;
    //         }
    //     })
    // }
}
