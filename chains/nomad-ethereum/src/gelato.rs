use ethers::providers::Middleware;
use gelato_relay::{GelatoClient, RelayResponse, TaskState};
use nomad_core::Signers;
use std::marker::PhantomData;

/*
{
  chainId: number;
  target: string; ** contract address?
  data: BytesLike;
  feeToken: string;
  paymentType: number; ** some kind of enum for gas tank vs. legacy?
  maxFee: string; ** just call get_estimated_fee?
  sponsor: string;
  sponsorChainId: number;
  nonce: number; ** does this enforce ordering too?
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
    /// Base client
    pub client: GelatoClient,
    /// Sponsor signer
    pub sponsor: Signers,
    /// Chain id
    pub chain_id: usize,
    /// Fee token
    pub fee_token: String,
    /// Transactions are of high priority
    pub is_high_priority: bool,
    /// Unused
    _middleware: PhantomData<M>,
}

impl<M: Middleware + 'static> SingleChainGelatoClient<M> {
    /// Get reference to base client
    pub fn client(&self) -> &GelatoClient {
        &self.client
    }

    /// Instantiate single chain client with default Gelato url
    pub fn with_default_url(
        sponsor: Signers,
        chain_id: usize,
        fee_token: String,
        is_high_priority: bool,
    ) -> Self {
        Self {
            client: GelatoClient::default(),
            sponsor,
            chain_id,
            fee_token,
            is_high_priority,
            _middleware: Default::default(),
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
            .client
            .get_estimated_fee(
                self.chain_id,
                &self.fee_token,
                gas_limit,
                self.is_high_priority,
            )
            .await?;

        self.client
            .send_relay_transaction(self.chain_id, dest, data, &self.fee_token, relayer_fee)
            .await
    }
}
