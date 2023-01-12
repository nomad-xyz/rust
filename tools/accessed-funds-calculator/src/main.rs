use std::collections::HashMap;
use std::env;

mod tokens;

use crate::tokens::{Token, TokenName};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    let usdc = Token {
        name: TokenName::USDC,
        contract_address: String::from("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
    };

    let cqt = Token {
        name: TokenName::CQT,
        contract_address: String::from("0xD417144312DbF50465b1C641d016962017Ef6240"),
    };

    let usdt = Token{
        name: TokenName::WBTC,
        contract_address: String::from("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"),
    };

    let wbtc = Token {
        name: TokenName::FRAX,
        contract_address: String::from("0x853d955aCEf822Db058eb8505911ED77F175b99e"),
    };

    let frax = Token {
        name: TokenName::FRAX,
        contract_address: String::from("0x853d955aCEf822Db058eb8505911ED77F175b99e"),
    };

    let iag = Token {
        name: TokenName::IAG,
        contract_address: String::from("0x40EB746DEE876aC1E78697b7Ca85142D178A1Fc8"),
    };

    let weth = Token {
        name: TokenName::WETH,
        contract_address: String::from("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
    };

    let dai = Token {
        name: TokenName::DAI,
        contract_address: String::from("0x6B175474E89094C44Da98b954EedeAC495271d0F"),
    };

    let c3 = Token {
        name: TokenName::C3,
        contract_address: String::from("0xf1a91C7d44768070F711c68f33A7CA25c8D30268"),
    };

    let fxs = Token {
        name: TokenName::FXS,
        contract_address: String::from("0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0"),
    };

    let cards = Token {
        name: TokenName::CARDS,
        contract_address: String::from("0x3d6F0DEa3AC3C607B3998e6Ce14b6350721752d9"),
    };

    let hbot = Token {
        name: TokenName::HBOT,
        contract_address: String::from("0xE5097D9baeAFB89f9bcB78C9290d545dB5f9e9CB"),
    };

    let sdl = Token {
        name: TokenName::SDL,
        contract_address: String::from("0xf1Dc500FdE233A4055e25e5BbF516372BC4F6871"),
    };

    let gero = Token {
        name: TokenName::GERO,
        contract_address: String::from("0x3431F91b3a388115F00C5Ba9FdB899851D005Fb5"),
    };

    let address = "0xa4B86BcbB18639D8e708d6163a0c734aFcDB770c";

    let balance_usdc: u128 = get_token_balance(&usdc, address).await?["result"].parse().unwrap();
    let balance_cqt: u128 = get_token_balance(&cqt, address).await?["result"].parse().unwrap();
    let balance_usdt: u128 = get_token_balance(&usdt, address).await?["result"].parse().unwrap();
    let balance_wbtc: u128 = get_token_balance(&wbtc, address).await?["result"].parse().unwrap();
    let balance_frax: u128 = get_token_balance(&frax, address).await?["result"].parse().unwrap();
    let balance_iag: u128 = get_token_balance(&iag, address).await?["result"].parse().unwrap();
    let balance_weth: u128 = get_token_balance(&weth, address).await?["result"].parse().unwrap();
    let balance_dai: u128 = get_token_balance(&dai, address).await?["result"].parse().unwrap();
    let balance_c3: u128 = get_token_balance(&c3, address).await?["result"].parse().unwrap();
    let balance_fxs: u128 = get_token_balance(&fxs, address).await?["result"].parse().unwrap();
    let balance_cards: u128 = get_token_balance(&cards, address).await?["result"].parse().unwrap();
    let balance_hbot: u128 = get_token_balance(&hbot, address).await?["result"].parse().unwrap();
    let balance_sdl: u128 = get_token_balance(&sdl, address).await?["result"].parse().unwrap();
    let balance_gero: u128 = get_token_balance(&gero, address).await?["result"].parse().unwrap();
    
    let balances = vec![
        balance_usdc,
        balance_cqt,
        balance_usdt,
        balance_wbtc,
        balance_frax,
        balance_iag,
        balance_weth,
        balance_dai,
        balance_c3,
        balance_fxs,
        balance_cards,
        balance_hbot,
        balance_sdl,
        balance_gero,
    ];

    for balance in balances {
        println!("{}", balance );
    }
    Ok(())
}

async fn get_token_balance(token: &Token, address: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let etherscan_url = "https://api.etherscan.io/api";
    let module = "account";
    let action = "tokenbalance";
    let apiKey = env::var("ETHERSCAN_KEY")?;
    let request_url = format!("{}?module={}&action={}&contractaddress={}&address={}&apiKey={}", &etherscan_url, &module, &action, &token.contract_address, &address, &apiKey);        
    let resp = reqwest::get(request_url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    Ok(resp)
}
