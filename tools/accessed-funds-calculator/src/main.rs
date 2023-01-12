use std::collections::HashMap;
use std::env;

use serde::{Deserialize, Serialize};

mod tokens;

use crate::tokens::{Token, TokenName};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    let usdc = Token {
        name: TokenName::USDC,
        id: String::from("usd-coin"),
        contract_address: String::from("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
    };

    let cqt = Token {
        name: TokenName::CQT,
        id: String::from("covalent"),
        contract_address: String::from("0xD417144312DbF50465b1C641d016962017Ef6240"),
    };

    let usdt = Token{
        name: TokenName::WBTC,
        id: String::from("tether"),
        contract_address: String::from("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"),
    };

    let wbtc = Token {
        name: TokenName::FRAX,
        id: String::from("wrapped-bitcoin"),
        contract_address: String::from("0x853d955aCEf822Db058eb8505911ED77F175b99e"),
    };

    let frax = Token {
        name: TokenName::FRAX,
        id: String::from("frax"),
        contract_address: String::from("0x853d955aCEf822Db058eb8505911ED77F175b99e"),
    };

    let iag = Token {
        name: TokenName::IAG,
        id: String::from("iagon"),
        contract_address: String::from("0x40EB746DEE876aC1E78697b7Ca85142D178A1Fc8"),
    };

    let weth = Token {
        name: TokenName::WETH,
        id: String::from("weth"),
        contract_address: String::from("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
    };

    let dai = Token {
        name: TokenName::DAI,
        id: String::from("dai"),
        contract_address: String::from("0x6B175474E89094C44Da98b954EedeAC495271d0F"),
    };

    let c3 = Token {
        name: TokenName::C3,
        id: String::from("charli3"),
        contract_address: String::from("0xf1a91C7d44768070F711c68f33A7CA25c8D30268"),
    };

    let fxs = Token {
        name: TokenName::FXS,
        id: String::from("frax-share"),
        contract_address: String::from("0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0"),
    };

    let cards = Token {
        name: TokenName::CARDS,
        id: String::from("cardstarter"),
        contract_address: String::from("0x3d6F0DEa3AC3C607B3998e6Ce14b6350721752d9"),
    };

    let hbot = Token {
        name: TokenName::HBOT,
        id: String::from("hummingbot"),
        contract_address: String::from("0xE5097D9baeAFB89f9bcB78C9290d545dB5f9e9CB"),
    };

    let sdl = Token {
        name: TokenName::SDL,
        id: String::from("saddle-finance"),
        contract_address: String::from("0xf1Dc500FdE233A4055e25e5BbF516372BC4F6871"),
    };

    let gero = Token {
        name: TokenName::GERO,
        id: String::from("gerowallet"),
        contract_address: String::from("0x3431F91b3a388115F00C5Ba9FdB899851D005Fb5"),
    };

    let address = "0xa4B86BcbB18639D8e708d6163a0c734aFcDB770c";

    let token_vec = vec![
        usdc,
        cqt,
        usdt,
        wbtc,
        frax,
        iag,
        weth,
        dai,
        c3,
        fxs,
        cards,
        hbot,
        sdl,
        gero,
    ];

    for token in token_vec {
        let balance: u128 = get_token_balance(&token, address).await?;
        // let token_price = get_token_price(&token).await?;
        println!("{:?}",balance);
        // println!("{:#?}",token_price);
    }

    Ok(())
}

async fn get_token_balance(token: &Token, address: &str) -> Result<u128, Box<dyn std::error::Error>> {
    let etherscan_url = "https://api.etherscan.io/api";
    let module = "account";
    let action = "tokenbalance";
    let apiKey = env::var("ETHERSCAN_KEY")?;
    let request_url = format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token.contract_address, &address, &apiKey);        
    let resp = reqwest::get(request_url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    let balance = resp["result"].parse().unwrap();
    Ok(balance)
}

async fn get_token_price(token: &Token) -> Result<String, Box<dyn std::error::Error>> {
    let coingecko_url = "https://api.coingecko.com/api/v3/simple/price";
    let ids = "ethereum";
    let vs_currency = "usd";

    let request_url = format!("{}?ids={}&vs_currencies={}", coingecko_url, ids, vs_currency);

    let resp = reqwest::get(request_url)
    .await?
    .json::<String>()
    .await?;
    let price = resp;
    println!("here");
    Ok(price)

}
