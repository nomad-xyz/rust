use std::collections::HashMap;
use std::env;

mod tokens;

use crate::tokens::{Token, TokenName};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "0xa4B86BcbB18639D8e708d6163a0c734aFcDB770c";
    let usdc = Token::get_instance_of(TokenName::Usdc);
    let usdt = Token::get_instance_of(TokenName::Usdt);
    let cqt = Token::get_instance_of(TokenName::Cqt);
    let wbtc = Token::get_instance_of(TokenName::Wbtc);
    let frax = Token::get_instance_of(TokenName::Frax);
    let iag = Token::get_instance_of(TokenName::Iag);
    let weth = Token::get_instance_of(TokenName::Weth);
    let dai = Token::get_instance_of(TokenName::Dai);
    let c3 = Token::get_instance_of(TokenName::C3);
    let fxs = Token::get_instance_of(TokenName::Fxs);
    let cards = Token::get_instance_of(TokenName::Cards);
    let hbot = Token::get_instance_of(TokenName::Hbot);
    let sdl = Token::get_instance_of(TokenName::Sdl);
    let gero = Token::get_instance_of(TokenName::Gero);

    let tokens: Vec<Token> = vec![
        usdc, cqt, usdt, wbtc, frax, iag, weth, dai, c3, fxs, cards, hbot, sdl, gero,
    ];

    let mut total_accessed_value: f64 = 0.0;
    for token in tokens {
        let balance: f64 = get_token_balance(&token, address).await?;
        let accessed: f64 = token.recovered_total - balance;
        let token_price: f64 = get_token_price(&token).await?;
        total_accessed_value += accessed * token_price;
        println!("{}:{} accessed, price: {}", token.id, accessed, token_price);
    }
    
    println!();
    println!("#################################################");
    println!("#################################################");
    println!("### total accessed value: ${} ###", total_accessed_value);
    println!("#################################################");
    println!("#################################################");

    Ok(())
}

async fn get_token_balance(
    token: &Token,
    address: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
    let etherscan_url = "https://api.etherscan.io/api";
    let module= "account";
    let action = "tokenbalance";
    let api_key = env::var("ETHERSCAN_KEY")?;
    let request_url = format!(
        "{}?module={}&action={}&contractaddress={}&address={}&apiKey={}",
        &etherscan_url, &module, &action, &token.contract_address, &address, &api_key
    );
    let resp = reqwest::get(request_url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    let balance: f64 = resp["result"].parse().unwrap();
    let balance = balance / ((10.0_f64).powf(token.decimals));

    Ok(balance)
}

async fn get_token_price(token: &Token) -> Result<f64, Box<dyn std::error::Error>> {
    let coingecko_url = "https://api.coingecko.com/api/v3/simple/price";
    let vs_currency = "usd";

    let request_url = format!(
        "{}?ids={}&vs_currencies={}",
        coingecko_url, &token.id, vs_currency
    );
    let resp = reqwest::get(request_url)
        .await?
        .json::<HashMap<String, HashMap<String, f64>>>()
        .await?;
    let price = resp[&token.id].clone();
    let price = price["usd"];

    Ok(price)
}
