#[derive(Debug)]
pub enum TokenName {
    Usdc,
    Cqt,
    Usdt,
    Frax,
    Wbtc,
    Iag,
    Weth,
    Dai,
    C3,
    Fxs,
    Cards,
    Hbot,
    Sdl,
    Gero,
}

#[derive(Debug)]
pub struct Token {
    pub name: TokenName,
    pub id: String,
    pub decimals: f64,
    pub contract_address: String,
    pub recovered_total: f64,
}
