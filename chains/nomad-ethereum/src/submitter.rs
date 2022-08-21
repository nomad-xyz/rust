use crate::{EthereumError, SingleChainGelatoClient};
use color_eyre::Result;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use nomad_core::{ChainCommunicationError, TxOutcome};
use std::sync::Arc;

/// Component responsible for submitting transactions to the chain. Can
/// sign/submit locally or use a transaction relay service.
#[derive(Debug, Clone)]
pub enum SubmitterClient<M> {
    /// Sign/submit txs locally
    Local(Arc<M>),
    /// Pass meta txs to Gelato relay service
    Gelato(Arc<SingleChainGelatoClient<M>>),
}

impl<M> From<Arc<M>> for SubmitterClient<M> {
    fn from(client: Arc<M>) -> Self {
        Self::Local(client)
    }
}

impl<M> From<SingleChainGelatoClient<M>> for SubmitterClient<M> {
    fn from(client: SingleChainGelatoClient<M>) -> Self {
        Self::Gelato(client.into())
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
    ) -> Result<TxOutcome, EthereumError> {
        let tx: TypedTransaction = tx.into();

        match &self.client {
            SubmitterClient::Local(client) => report_tx!(tx, client,).try_into(),
            SubmitterClient::Gelato(client) => {
                client.submit_blocking(domain, contract_address, &tx).await
            }
        }
    }
}
