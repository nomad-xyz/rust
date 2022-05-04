use ethers::providers::Middleware;
use ethers::types::Address;
use ethers_signers::Signer;
use gelato_relay::{ForwardRequest, GelatoClient, RelayResponse, TaskState};
use nomad_core::{ChainCommunicationError, Signers};
use std::sync::Arc;

/// Sponsor data encoding/signing
mod sponsor;
pub use sponsor::*;

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
        target: &str,
        data: &str,
        gas_limit: usize,
    ) -> Result<RelayResponse, ChainCommunicationError> {
        let max_fee = self
            .gelato()
            .get_estimated_fee(self.chain_id, &self.fee_token, gas_limit, false)
            .await
            .map_err(|e| ChainCommunicationError::CustomError(e.into()))?;

        let mut request = ForwardRequest {
            chain_id: self.chain_id,
            target: target.to_owned(),
            data: data.to_owned(),
            fee_token: self.fee_token.to_owned(),
            payment_type: 1, // gas tank
            max_fee: max_fee.to_string(),
            sponsor: format!("{:x}", self.sponsor.address()),
            sponsor_chain_id: self.chain_id,
            nonce: 0,                     // default, not needed
            enforce_sponsor_nonce: false, // replay safety builtin to contracts
            sponsor_signature: None,      // not yet signed
        };

        let sponsor_signature = sponsor_sign_request(&self.sponsor, self.forwarder, &request)
            .await
            .map_err(|e| ChainCommunicationError::CustomError(e.into()))?;

        request.sponsor_signature = Some(sponsor_signature);

        Ok(RelayResponse {
            task_id: "id".to_owned(),
        }) // TODO: replace with call to endpoint
    }
}
