use crate::{GelatoClient, RelayResponse};
use ethers::providers::Middleware;
use std::marker::PhantomData;

/// Gelato client for submitting txs to single chain
#[derive(Debug, Clone)]
pub struct SingleChainGelatoClient<M> {
    pub client: GelatoClient,
    pub chain_id: usize,
    pub payment_token: String,
    _middleware: PhantomData<M>,
}

impl<M: Middleware + 'static> SingleChainGelatoClient<M> {
    /// Get reference to base client
    pub fn client(&self) -> &GelatoClient {
        &self.client
    }

    /// Instantiate single chain client with default Gelato url
    pub fn with_default_url(chain_id: usize, payment_token: String) -> Self {
        Self {
            client: GelatoClient::default(),
            chain_id,
            payment_token,
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
            .get_estimated_fee(self.chain_id, &self.payment_token, 100_000, true)
            .await?;

        self.client
            .send_relay_transaction(self.chain_id, dest, data, &self.payment_token, relayer_fee)
            .await
    }
}
