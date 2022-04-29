use crate::{GelatoClient, RelayResponse};

/// Gelato client for submitting txs to single chain
pub struct SingleChainGelatoClient {
    pub client: GelatoClient,
    pub chain_id: usize,
    pub payment_token: String,
}

impl SingleChainGelatoClient {
    /// Instantiate single chain client with default Gelato url
    pub fn with_default_url(chain_id: usize, payment_token: String) -> Self {
        Self {
            client: GelatoClient::default(),
            chain_id,
            payment_token,
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
