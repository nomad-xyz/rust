use ethers::{
    prelude::{Address, Bytes, H256, U64},
    providers::Middleware,
    types::transaction::eip2718::TypedTransaction,
};
use gelato_sdk::{
    get_forwarder,
    rpc::{RelayResponse, TaskState},
    FeeToken, ForwardRequestBuilder, GelatoClient,
};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::info;

use nomad_core::{ChainCommunicationError, Signers, TxOutcome};

pub(crate) const ACCEPTABLE_STATES: [TaskState; 4] = [
    TaskState::CheckPending,
    TaskState::ExecPending,
    TaskState::ExecSuccess,
    TaskState::WaitingForConfirmation,
];

/// Gelato client for submitting txs to single chain
#[derive(Debug, Clone)]
pub struct SingleChainGelatoClient<M> {
    /// Gelato client
    pub gelato: Arc<GelatoClient>,
    /// Ethers client (for estimating gas)
    pub eth_client: Arc<M>,
    /// Sponsor signer
    pub sponsor: Signers,
    /// Gelato relay forwarder address
    pub forwarder: Address,
    /// Chain id
    pub chain_id: u64,
    /// Fee token
    pub fee_token: FeeToken,
    /// Transactions are of high priority
    pub is_high_priority: bool,
}

impl<M> SingleChainGelatoClient<M>
where
    M: Middleware + 'static,
{
    /// Get reference to base client
    pub fn gelato(&self) -> Arc<GelatoClient> {
        self.gelato.clone()
    }

    /// Instantiate single chain client with default Gelato url
    pub fn with_default_url(
        eth_client: Arc<M>,
        sponsor: Signers,
        chain_id: u64,
        fee_token: impl Into<FeeToken>,
        is_high_priority: bool,
    ) -> Self {
        Self {
            gelato: GelatoClient::default().into(),
            eth_client,
            sponsor,
            forwarder: get_forwarder(chain_id).expect("!forwarder proxy"),
            chain_id,
            fee_token: fee_token.into(),
            is_high_priority,
        }
    }

    /// Submit a transaction to Gelato and poll until completion or failure.
    pub async fn submit_blocking(
        &self,
        domain: u32,
        contract_address: Address,
        tx: &TypedTransaction,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        let task_id = self
            .dispatch_tx(domain, contract_address, tx)
            .await?
            .task_id();

        info!(task_id = ?&task_id, "Submitted tx to Gelato relay.");

        info!(task_id = ?&task_id, "Polling Gelato task...");
        self.poll_task_id(task_id)
            .await
            .map_err(|e| ChainCommunicationError::TxSubmissionError(e.into()))?
    }

    /// Dispatch tx to Gelato and return task id.
    pub async fn dispatch_tx(
        &self,
        domain: u32,
        contract_address: Address,
        tx: &TypedTransaction,
    ) -> Result<RelayResponse, ChainCommunicationError> {
        // If gas limit not hardcoded in tx, eth_estimateGas
        let gas_limit = tx
            .gas()
            .unwrap_or(
                &self
                    .eth_client
                    .estimate_gas(tx)
                    .await
                    .map_err(|e| ChainCommunicationError::MiddlewareError(e.into()))?,
            )
            .as_u64()
            .into();
        let data = tx.data().cloned().unwrap_or_default();

        info!(
            domain = domain,
            contract_address = ?contract_address,
            "Dispatching tx to Gelato relay."
        );

        self.send_forward_request(contract_address, data, gas_limit)
            .await
            .map_err(|e| ChainCommunicationError::TxSubmissionError(e.into()))
    }

    /// Poll task id and return tx hash of transaction if successful, error if
    /// otherwise.
    pub fn poll_task_id(
        &self,
        task_id: H256,
    ) -> JoinHandle<Result<TxOutcome, ChainCommunicationError>> {
        let gelato = self.gelato();

        tokio::spawn(async move {
            loop {
                let status = gelato
                    .get_task_status(task_id)
                    .await
                    .map_err(|e| ChainCommunicationError::TxSubmissionError(e.into()))?
                    .expect("!task status");

                if !ACCEPTABLE_STATES.contains(&status.task_state) {
                    return Err(ChainCommunicationError::TxSubmissionError(
                        format!("Gelato task failed: {:?}", status).into(),
                    ));
                }

                if let Some(execution) = &status.execution {
                    info!(
                        chain = ?status.chain,
                        task_id = ?status.task_id,
                        execution = ?execution,
                        "Gelato relay executed tx."
                    );

                    let txid = execution.transaction_hash;

                    return Ok(TxOutcome { txid });
                }

                if status.task_state == TaskState::CheckPending {
                    info!(status = ?status, "Polling pending Gelato task...");
                }

                sleep(Duration::from_secs(3)).await;
            }
        })
    }

    /// Format and sign forward request, then dispatch to Gelato relay service.
    ///
    /// This function pads gas by 100k to allow for gelato ops
    pub async fn send_forward_request(
        &self,
        target: Address,
        data: impl Into<Bytes>,
        gas_limit: U64,
    ) -> Result<RelayResponse, ChainCommunicationError> {
        // add 100k gas padding for Gelato contract ops
        let adjusted_limit = gas_limit + U64::from(100_000);

        let max_fee = self
            .gelato()
            .get_estimated_fee(self.chain_id, self.fee_token, adjusted_limit, false)
            .await
            .map_err(|e| ChainCommunicationError::CustomError(e.into()))?;

        let request = ForwardRequestBuilder::default()
            .chain_id(self.chain_id)
            .target(target)
            .data(data.into())
            .fee_token(self.fee_token)
            .max_fee(max_fee)
            .gas(gas_limit)
            .sponsored_by(&self.sponsor)
            .sponsor_chain_id(self.chain_id)
            .enforce_sponsor_nonce(false)
            .build()
            .await
            .expect("signer doesn't fail");

        info!(request = ?request, "Signed gelato forward request.");

        self.gelato()
            .send_forward_request(&request)
            .await
            .map_err(|e| ChainCommunicationError::TxSubmissionError(e.into()))
    }
}
