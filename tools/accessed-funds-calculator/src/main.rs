use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query_list = vec![
        AffectedTokens::USDC(String::from("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")),
        AffectedTokens::CQT(String::from("0xD417144312DbF50465b1C641d016962017Ef6240")),
        AffectedTokens::USDT(String::from("0xdAC17F958D2ee523a2206206994597C13D831ec7")),
        AffectedTokens::FRAX(String::from("0x853d955aCEf822Db058eb8505911ED77F175b99e")),
        AffectedTokens::WBTC(String::from("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599")),
        AffectedTokens::IAG(String::from("0x40EB746DEE876aC1E78697b7Ca85142D178A1Fc8")),
        AffectedTokens::WETH(String::from("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")),
        AffectedTokens::DAI(String::from("0x6B175474E89094C44Da98b954EedeAC495271d0F")),
        AffectedTokens::MC3(String::from("0xf1a91C7d44768070F711c68f33A7CA25c8D30268")),
        AffectedTokens::FXS(String::from("0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0")),
        AffectedTokens::CARDS(String::from("0x3d6F0DEa3AC3C607B3998e6Ce14b6350721752d9")),
        AffectedTokens::HBOT(String::from("0xE5097D9baeAFB89f9bcB78C9290d545dB5f9e9CB")),
        AffectedTokens::SDL(String::from("0xf1Dc500FdE233A4055e25e5BbF516372BC4F6871")),
        AffectedTokens::GERO(String::from("0x3431F91b3a388115F00C5Ba9FdB899851D005Fb5")),
    ];
    let address = "0xa4B86BcbB18639D8e708d6163a0c734aFcDB770c";

    for token in query_list.iter() {
        let value: u128 = get_token_balance(&token, address).await?["result"].parse().unwrap();
        println!("{:?}", value);
    }
    Ok(())
}

async fn get_token_balance(token: &AffectedTokens, address: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let etherscan_url = "https://api.etherscan.io/api";
    let module = "account";
    let action = "tokenbalance";
    let apiKey = env::var("ETHERSCAN_KEY")?;
    let request_url = 
        match token {
            AffectedTokens::USDC(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::CQT(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::USDT(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::FRAX(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::WBTC(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::IAG(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::WETH(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::DAI(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::MC3(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::FXS(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::CARDS(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::HBOT(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::SDL(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
            AffectedTokens::GERO(token_address) => format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token_address, &address, &apiKey),
    };
        
    let resp = reqwest::get(request_url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    Ok(resp)
}

enum AffectedTokens {
    USDC(String),
    CQT(String),
    USDT(String),
    FRAX(String),
    WBTC(String),
    IAG(String),
    WETH(String),
    DAI(String),
    MC3(String),
    FXS(String),
    CARDS(String),
    HBOT(String),
    SDL(String),
    GERO(String),
}