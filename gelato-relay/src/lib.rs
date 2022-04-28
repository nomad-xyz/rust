mod types;
use types::*;

use color_eyre::eyre::Result;

const RELAY_URL: &str = "https://relay.gelato.digital";

pub async fn send_relay_transaction(
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

    let url = format!("{}/relays/{}", RELAY_URL, chain_id);
    let res = reqwest::Client::new()
        .post(url)
        .json(&params)
        .send()
        .await?;

    res.json().await
}

pub async fn is_chain_supported(chain_id: usize) -> Result<bool, reqwest::Error> {
    let supported_chains = get_gelato_relay_chains().await?;
    Ok(supported_chains.contains(&chain_id.to_string()))
}

pub async fn get_gelato_relay_chains() -> Result<Vec<String>, reqwest::Error> {
    let url = format!("{}/relays", RELAY_URL);
    let res = reqwest::get(url).await?;
    Ok(res.json::<RelayChainsResponse>().await?.relays)
}

pub async fn get_task_status(task_id: &str) -> Result<TaskStatusResponse, reqwest::Error> {
    let url = format!("{}/tasks/{}", RELAY_URL, task_id);
    let res = reqwest::get(url).await?;
    Ok(res.json().await?)
}
