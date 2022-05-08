use ethers::signers::Signer;
use ethers::types::Address;
use ethers::{prelude::Bytes, providers::Middleware};
use gelato_relay::{GelatoClient, RelayResponse, TaskState};
use nomad_core::{ChainCommunicationError, Signers};
use std::sync::Arc;

/// EIP-712 forward request structure
mod types;
pub use types::*;

pub(crate) const FORWARD_REQUEST_TYPE_ID: &str = "ForwardRequest";

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
    pub gelato: GelatoClient,
    /// Ethers client (for estimating gas)
    pub eth_client: Arc<M>,
    /// Sponsor signer
    pub sponsor: Signers,
    /// Gelato relay forwarder address
    pub forwarder: Address,
    /// Chain id
    pub chain_id: usize,
    /// Fee token
    pub fee_token: String,
    /// Transactions are of high priority
    pub is_high_priority: bool,
}

impl<M> SingleChainGelatoClient<M>
where
    M: Middleware + 'static,
{
    /// Get reference to base client
    pub fn gelato(&self) -> &GelatoClient {
        &self.gelato
    }

    /// Instantiate single chain client with default Gelato url
    pub fn with_default_url(
        eth_client: Arc<M>,
        sponsor: Signers,
        forwarder: Address,
        chain_id: usize,
        fee_token: String,
        is_high_priority: bool,
    ) -> Self {
        Self {
            gelato: GelatoClient::default(),
            eth_client,
            sponsor,
            forwarder,
            chain_id,
            fee_token,
            is_high_priority,
        }
    }

    /// Send relay transaction
    pub async fn send_forward_request(
        &self,
        target: Address,
        data: &Bytes,
        _gas_limit: usize,
    ) -> Result<RelayResponse, ChainCommunicationError> {
        // let estimated_fee = self
        //     .gelato()
        //     .get_estimated_fee(self.chain_id, &self.fee_token, gas_limit, false)
        //     .await
        //     .map_err(|e| ChainCommunicationError::CustomError(e.into()))?;

        let max_fee = 10000;

        let target = format!("{:#x}", target);
        let sponsor = format!("{:#x}", self.sponsor.address());

        let unfilled_request = UnfilledFowardRequest {
            type_id: FORWARD_REQUEST_TYPE_ID.to_owned(),
            chain_id: self.chain_id,
            target,
            data: data.to_string(),
            fee_token: self.fee_token.to_owned(),
            payment_type: 1, // gas tank
            max_fee,
            sponsor,
            sponsor_chain_id: self.chain_id,
            nonce: 0,                     // default, not needed
            enforce_sponsor_nonce: false, // replay safety builtin to contracts
        };

        let sponsor_signature = self
            .sponsor
            .sign_typed_data(&unfilled_request)
            .await
            .unwrap();

        let filled_request = unfilled_request.into_filled(sponsor_signature.to_vec());

        self.gelato()
            .send_forward_request(&filled_request)
            .await
            .map_err(|e| ChainCommunicationError::TxSubmissionError(e.into()))
    }
}
