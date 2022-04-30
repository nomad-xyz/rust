use ethers::providers::Middleware;
use gelato_relay::{GelatoClient, RelayResponse};
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
    /// Unused
    _middleware: PhantomData<M>,
}

impl<M: Middleware + 'static> SingleChainGelatoClient<M> {
    /// Get reference to base client
    pub fn client(&self) -> &GelatoClient {
        &self.client
    }

    /// Instantiate single chain client with default Gelato url
    pub fn with_default_url(sponsor: Signers, chain_id: usize, fee_token: String) -> Self {
        Self {
            client: GelatoClient::default(),
            sponsor,
            chain_id,
            fee_token,
            _middleware: Default::default(),
        }
    }

    /// Send relay transaction
    pub async fn send_relay_transaction(
        &self,
        dest: &str,
        data: &str,
    ) -> Result<RelayResponse, reqwest::Error> {
        let relayer_fee = self
            .client
            .get_estimated_fee(self.chain_id, &self.fee_token, 100_000, true)
            .await?;

        self.client
            .send_relay_transaction(self.chain_id, dest, data, &self.fee_token, relayer_fee)
            .await
    }
}
