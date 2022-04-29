use gelato_relay::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), reqwest::Error> {
    let gelato = GelatoClient::default();

    let chains = gelato.get_gelato_relay_chains().await?;
    println!("Relay chains: {:?}", chains);

    let task_status = gelato
        .get_task_status("0xeefc20b15402c30ead95d572034532c7097488726a6582d3d6674971e9d97879")
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
