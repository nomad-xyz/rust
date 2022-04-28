mod types;
use types::*;

use color_eyre::eyre::Result;

const DEFAULT_URL: &str = "https://relay.gelato.digital";

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
        relayer_fee: &str,
    ) -> Result<RelayResponse, reqwest::Error> {
        let params = RelayRequest {
            dest: dest.to_owned(),
            data: data.to_owned(),
            token: token.to_owned(),
            relayer_fee: relayer_fee.to_owned(),
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

    pub async fn get_task_status(
        &self,
        task_id: &str,
    ) -> Result<TaskStatusResponse, reqwest::Error> {
        let url = format!("{}/tasks/{}", &self.url, task_id);
        let res = reqwest::get(url).await?;
        Ok(res.json().await?)
    }
}
