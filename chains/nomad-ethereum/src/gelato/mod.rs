use ethers::providers::Middleware;
use gelato_relay::{GelatoClient, RelayResponse, TaskState};
use nomad_core::Signers;
use std::sync::Arc;

mod sponsor;
pub use sponsor::*;

/*
{
  chainId: number;
  target: string;
  data: BytesLike;
  feeToken: string;
  paymentType: number;
  maxFee: string;
  sponsor: string;
  sponsorChainId: number;
  nonce: number;
  enforceSponsorNonce: boolean;
  sponsorSignature: BytesLike;
}
 */

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
        chain_id: usize,
        fee_token: String,
        is_high_priority: bool,
    ) -> Self {
        Self {
            gelato: GelatoClient::default(),
            eth_client,
            sponsor,
            chain_id,
            fee_token,
            is_high_priority,
        }
    }

    /// Send relay transaction
    pub async fn send_relay_transaction(
        &self,
        dest: &str,
        data: &str,
        gas_limit: usize,
    ) -> Result<RelayResponse, reqwest::Error> {
        let relayer_fee = self
            .gelato()
            .get_estimated_fee(
                self.chain_id,
                &self.fee_token,
                gas_limit,
                self.is_high_priority,
            )
            .await?;

        self.gelato()
            .send_relay_transaction(self.chain_id, dest, data, &self.fee_token, relayer_fee)
            .await
    }
}
