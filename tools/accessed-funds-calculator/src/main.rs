use std::collections::HashMap;
use std::env;

mod tokens;

use crate::tokens::{Token, TokenName};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let usdc = Token {
        name: TokenName::Usdc,
        id: "usd-coin".to_string(),
        decimals: 6.0,
        contract_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        recovered_total: 12_890_538.932_401,
    };

    let cqt = Token {
        name: TokenName::Cqt,
        id: "covalent".to_string(),
        decimals: 18.0,
        contract_address: "0xD417144312DbF50465b1C641d016962017Ef6240".to_string(),
        recovered_total: 34_082_775.751_599_7,
    };

    let usdt = Token {
        name: TokenName::Usdt,
        id: "tether".to_string(),
        decimals: 6.0,
        contract_address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
        recovered_total: 4_673_863.595_197,
    };

    let wbtc = Token {
        name: TokenName::Wbtc,
        id: "wrapped-bitcoin".to_string(),
        decimals: 8.0,
        contract_address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string(),
        recovered_total: 280.731_173_99,
    };

    let frax = Token {
        name: TokenName::Frax,
        id: "frax".to_string(),
        decimals: 18.0,
        contract_address: "0x853d955aCEf822Db058eb8505911ED77F175b99e".to_string(),
        recovered_total: 2_644_469.918_609_09,
    };

    let iag = Token {
        name: TokenName::Iag,
        id: "iagon".to_string(),
        decimals: 18.0,
        contract_address: "0x40EB746DEE876aC1E78697b7Ca85142D178A1Fc8".to_string(),
        recovered_total: 349_507_392.187_402,
    };

    let weth = Token {
        name: TokenName::Weth,
        id: "weth".to_string(),
        decimals: 18.0,
        contract_address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        recovered_total: 1_049.635_629_8,
    };

    let dai = Token {
        name: TokenName::Dai,
        id: "dai".to_string(),
        decimals: 18.0,
        contract_address: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
        recovered_total: 866_070.756_876_35,
    };

    let c3 = Token {
        name: TokenName::C3,
        id: "charli3".to_string(),
        decimals: 18.0,
        contract_address: "0xf1a91C7d44768070F711c68f33A7CA25c8D30268".to_string(),
        recovered_total: 1_684_711.122_391_36,
    };

    let fxs = Token {
        name: TokenName::Fxs,
        id: "frax-share".to_string(),
        decimals: 18.0,
        contract_address: "0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0".to_string(),
        recovered_total: 46_895.688_044_5,
    };

    let cards = Token {
        name: TokenName::Cards,
        id: "cardstarter".to_string(),
        decimals: 18.0,
        contract_address: "0x3d6F0DEa3AC3C607B3998e6Ce14b6350721752d9".to_string(),
        recovered_total: 165_005.819_480_28,
    };

    let hbot = Token {
        name: TokenName::Hbot,
        id: "hummingbot".to_string(),
        decimals: 18.0,
        contract_address: "0xE5097D9baeAFB89f9bcB78C9290d545dB5f9e9CB".to_string(),
        recovered_total: 900_239.997_966,
    };

    let sdl = Token {
        name: TokenName::Sdl,
        id: "saddle-finance".to_string(),
        decimals: 18.0,
        contract_address: "0xf1Dc500FdE233A4055e25e5BbF516372BC4F6871".to_string(),
        recovered_total: 9_790.824_057,
    };

    let gero = Token {
        name: TokenName::Gero,
        id: "gerowallet".to_string(),
        decimals: 18.0,
        contract_address: "0x3431F91b3a388115F00C5Ba9FdB899851D005Fb5".to_string(),
        recovered_total: 23_245_641.666_183_1,
    };

    let address = "0xa4B86BcbB18639D8e708d6163a0c734aFcDB770c";

    let token_vec = vec![
        usdc, cqt, usdt, wbtc, frax, iag, weth, dai, c3, fxs, cards, hbot, sdl, gero,
    ];

    let mut total_accessed_value = 0.0;
    for token in token_vec {
        let balance: f64 = get_token_balance(&token, address).await?;
        let accessed = token.recovered_total - balance;
        let token_price = get_token_price(&token).await?;
        total_accessed_value += accessed * token_price;
        
        println!("{}:{} accessed, price: {}", token.id, accessed, token_price);
    }

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
    let module = "account";
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
