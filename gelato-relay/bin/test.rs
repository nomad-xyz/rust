use gelato_relay::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), reqwest::Error> {
    let gelato = GelatoClient::default();

    let chains = gelato.get_gelato_relay_chains().await?;
    println!("Relay chains: {:?}", chains);

    let task_status = gelato
        .get_task_status("0xce52ae7a6a3032848d76b161ac4c131fa995dcc67e3be5392dfb8466275d6679")
        .await?;
    println!("Task status: {:?}", task_status);

    let mainnet: usize = chains[0].parse().unwrap();
    let estimated_fee = gelato
        .get_estimated_fee(
            mainnet,
            "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            100_000,
            true,
        )
        .await?;
    println!("Estimated fee: {:?}", estimated_fee);

    Ok(())
}
