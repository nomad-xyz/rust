mod types;
pub use types::*;

mod client;
pub use client::*;

use color_eyre::eyre::Result;
use std::collections::HashMap;

const DEFAULT_URL: &str = "https://relay.gelato.digital";

#[derive(Debug, Clone)]
pub struct GelatoClient {
    url: String,
}

impl Default for GelatoClient {
    fn default() -> Self {
        Self::new(DEFAULT_URL.to_owned())
    }
}

impl GelatoClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn send_relay_transaction(
        &self,
        chain_id: usize,
        dest: &str,
        data: &str,
        token: &str,
        relayer_fee: usize,
    ) -> Result<RelayResponse, reqwest::Error> {
        let params = RelayRequest {
            dest: dest.to_owned(),
            data: data.to_owned(),
            token: token.to_owned(),
            relayer_fee: relayer_fee.to_string(),
        };

        let url = format!("{}/relays/{}", &self.url, chain_id);
        let res = reqwest::Client::new()
            .post(url)
            .json(&params)
            .send()
            .await?;

        res.json().await
    }

    pub async fn is_chain_supported(&self, chain_id: usize) -> Result<bool, reqwest::Error> {
        let supported_chains = self.get_gelato_relay_chains().await?;
        Ok(supported_chains.contains(&chain_id.to_string()))
    }

    pub async fn get_gelato_relay_chains(&self) -> Result<Vec<String>, reqwest::Error> {
        let url = format!("{}/relays", &self.url);
        let res = reqwest::get(url).await?;
        Ok(res.json::<RelayChainsResponse>().await?.relays)
    }

    pub async fn get_estimated_fee(
        &self,
        chain_id: usize,
        payment_token: &str,
        gas_limit: usize,
        is_high_priority: bool,
    ) -> Result<usize, reqwest::Error> {
        let payment_token = payment_token.to_string();
        let gas_limit = gas_limit.to_string();
        let is_high_priority = is_high_priority.to_string();
        let params = HashMap::from([
            ("paymentToken", payment_token),
            ("gasLimit", gas_limit),
            ("isHighPriority", is_high_priority),
        ]);

        let base_url = format!("{}/oracles/{}/estimate", &self.url, chain_id);
        let url = reqwest::Url::parse_with_params(&base_url, params).expect("!url");
        let res = reqwest::get(url).await?;

        Ok(res
            .json::<EstimatedFeeResponse>()
            .await?
            .estimated_fee
            .parse()
            .expect("!string to int"))
    }

    pub async fn get_task_status(
        &self,
        task_id: &str,
    ) -> Result<Option<TaskStatus>, reqwest::Error> {
        let url = format!("{}/tasks/{}", &self.url, task_id);
        let res = reqwest::get(url).await?;
        let task_status: TaskStatusResponse = res.json().await?;
        Ok(task_status.data.first().cloned())
    }
}
