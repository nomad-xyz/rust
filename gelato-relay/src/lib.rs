mod types;
use types::*;

use color_eyre::eyre::Result;

const RELAY_URL: &str = "https://relay.gelato.digital";

pub async fn send_relay_transaction(
    chain_id: usize,
    dest: String,
    data: String,
    token: String,
    relayer_fee: String,
) -> Result<RelayTransaction> {
    let params = RelayRequest {
        dest,
        data,
        token,
        relayer_fee,
    };

    let client = reqwest::Client::new();
    let url = format!("{}/relays/{}", RELAY_URL, chain_id);
    let res = client
        .post(url)
        .json(&params)
        .send()
        .await?;

    res.json().await
}