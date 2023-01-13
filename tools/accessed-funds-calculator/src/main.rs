use std::collections::HashMap;
use std::env;

use serde_json;

mod tokens;

use crate::tokens::{Token, TokenName};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let usdc = Token {
        name: TokenName::USDC,
        id: String::from("usd-coin"),
        decimals: 6.0,
        contract_address: String::from("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
        recovered_total: 12_890_538.932401,
        currentPrice: 0.9992,
    };

    let cqt = Token {
        name: TokenName::CQT,
        id: String::from("covalent"),
        decimals: 18.0,
        contract_address: String::from("0xD417144312DbF50465b1C641d016962017Ef6240"),
        recovered_total: 34_082_775.75159970,
        currentPrice: 0.112701,
    };

    let usdt = Token {
        name: TokenName::USDT,
        id: String::from("tether"),
        decimals: 6.0,
        contract_address: String::from("0xdAC17F958D2ee523a2206206994597C13D831ec7"),
        recovered_total: 4_673_863.595197,
        currentPrice: 0.9994,
    };

    let wbtc = Token {
        name: TokenName::WBTC,
        id: String::from("wrapped-bitcoin"),
        decimals: 8.0,
        contract_address: String::from("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"),
        recovered_total: 280.73117399,
        currentPrice: 18_805.78,
    };

    let frax = Token {
        name: TokenName::FRAX,
        id: String::from("frax"),
        decimals: 18.0,
        contract_address: String::from("0x853d955aCEf822Db058eb8505911ED77F175b99e"),
        recovered_total: 2_644_469.91860909,
        currentPrice: 1.001,
    };

    let iag = Token {
        name: TokenName::IAG,
        id: String::from("iagon"),
        decimals: 18.0,
        contract_address: String::from("0x40EB746DEE876aC1E78697b7Ca85142D178A1Fc8"),
        recovered_total: 349_507_392.18740200,
        currentPrice: 0.0054,
    };

    let weth = Token {
        name: TokenName::WETH,
        id: String::from("weth"),
        decimals: 18.0,
        contract_address: String::from("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
        recovered_total: 1_049.63562980,
        currentPrice: 1425.31,
    };

    let dai = Token {
        name: TokenName::DAI,
        id: String::from("dai"),
        decimals: 18.0,
        contract_address: String::from("0x6B175474E89094C44Da98b954EedeAC495271d0F"),
        recovered_total: 866_070.75687635,
        currentPrice: 1.0,
    };

    let c3 = Token {
        name: TokenName::C3,
        id: String::from("charli3"),
        decimals: 18.0,
        contract_address: String::from("0xf1a91C7d44768070F711c68f33A7CA25c8D30268"),
        recovered_total: 1_684_711.12239136,
        currentPrice: 0.1673,
    };

    let fxs = Token {
        name: TokenName::FXS,
        id: String::from("frax-share"),
        decimals: 18.0,
        contract_address: String::from("0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0"),
        recovered_total: 46_895.68804450,
        currentPrice: 5.93,
    };

    let cards = Token {
        name: TokenName::CARDS,
        id: String::from("cardstarter"),
        decimals: 18.0,
        contract_address: String::from("0x3d6F0DEa3AC3C607B3998e6Ce14b6350721752d9"),
        recovered_total: 165_005.81948028,
        currentPrice: 0.2026,
    };

    let hbot = Token {
        name: TokenName::HBOT,
        id: String::from("hummingbot"),
        decimals: 18.0,
        contract_address: String::from("0xE5097D9baeAFB89f9bcB78C9290d545dB5f9e9CB"),
        recovered_total: 900_239.99796600,
        currentPrice: 0.0086,
    };

    let sdl = Token {
        name: TokenName::SDL,
        id: String::from("saddle-finance"),
        decimals: 18.0,
        contract_address: String::from("0xf1Dc500FdE233A4055e25e5BbF516372BC4F6871"),
        recovered_total: 9_790.82405700,
        currentPrice: 0.0,
    };

    let gero = Token {
        name: TokenName::GERO,
        id: String::from("gerowallet"),
        decimals: 18.0,
        contract_address: String::from("0x3431F91b3a388115F00C5Ba9FdB899851D005Fb5"),
        recovered_total: 23_245_641.66618310,
        currentPrice: 0.0,
    };

    let address = "0xa4B86BcbB18639D8e708d6163a0c734aFcDB770c";

    let token_vec = vec![
        usdc, cqt, usdt, wbtc, frax, iag, weth, dai, c3, fxs, cards, hbot,
    ];

    let mut total_accessed_value = 0.0;
    for token in token_vec {
        let balance: f64 = get_token_balance(&token, address).await?;
        let accessed = token.recovered_total - balance;
        total_accessed_value = total_accessed_value + (accessed * token.currentPrice);
        println!("{}:{}", token.id, accessed);

        // let token_price = get_token_price(&token).await?;
        // println!("{:#?}", token_price["ethereum"]);
    }

    println!("total accessed value: ${}", total_accessed_value);

    Ok(())
}

async fn get_token_balance(
    token: &Token,
    address: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
    let etherscan_url = "https://api.etherscan.io/api";
    let module = "account";
    let action = "tokenbalance";
    let apiKey = env::var("ETHERSCAN_KEY")?;
    let request_url = format!(
        "{}?module={}&action={}&contractaddress={}&address={}&apiKey={}",
        &etherscan_url, &module, &action, &token.contract_address, &address, &apiKey
    );
    let resp = reqwest::get(request_url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    let balance: f64 = resp["result"].parse().unwrap();
    let denominator = (10.0_f64).powf(token.decimals);
    // println!("balance: {}/{}", balance, denominator);
    let balance = balance / ((10.0_f64).powf(token.decimals));

    Ok(balance)
}

// async fn get_token_price(
//     token: &Token,
// ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
//     let coingecko_url = "https://api.coingecko.com/api/v3/simple/price";
//     let ids = "ethereum";
//     let vs_currency = "usd";

//     let request_url = format!(
//         "{}?ids={}&vs_currencies={}",
//         coingecko_url, ids, vs_currency
//     );

//     let resp = reqwest::get(request_url)
//     .await?
//     .text()
//     .await?;
//     // let resp: &str = &resp.await?;
//     let resp = serde_json::from_str(&resp)?;

//     dbg!(&resp);
//     // println!("{:?}",resp);
//     Ok(resp)
// }
