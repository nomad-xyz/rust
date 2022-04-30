use crate::http_signer_middleware;
use color_eyre::{eyre::bail, Result};
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use nomad_core::Signers;
use nomad_xyz_configuration::ethereum::Connection;
use std::sync::Arc;
use tokio::{sync::mpsc, task::JoinHandle};

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
pub enum Submitter {
    /// Sign/submit txs locally
    Local(SigningProviderConfig),
}

/// Receives meta txs and submits them to chain
#[derive(Debug)]
pub struct ChainSubmitter {
    /// Tx submitter
    pub submitter: Submitter,
    // /// Meta tx receiver
    // pub rx: mpsc::Receiver<MetaTx>,
}

impl ChainSubmitter {
    /// Submit transaction to chain
    pub async fn submit(
        &self,
        _domain: u32,
        _contract_address: Address,
        tx: impl Into<TypedTransaction>,
    ) -> Result<()> {
        let tx: TypedTransaction = tx.into();

        match &self.submitter {
            Submitter::Local(config) => {
                let signer = config.signer.clone();
                let client = match &config.connection {
                    Connection::Http { url } => http_signer_middleware!(url, signer),
                    Connection::Ws { url: _ } => panic!("not supporting ws"),
                };

                let dispatched = client.send_transaction(tx, None).await?;
                let tx_hash: ethers::core::types::H256 = *dispatched;
                let result = dispatched
                    .await?
                    .ok_or_else(|| nomad_core::ChainCommunicationError::DroppedError(tx_hash))?;

                tracing::info!(
                    "confirmed transaction with tx_hash {:?}",
                    result.transaction_hash
                );

                Ok(())
            }
        }
    }

    // /// Spawn ChainSubmitter task. Receives meta txs and submits in loop.
    // #[tracing::instrument]
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
