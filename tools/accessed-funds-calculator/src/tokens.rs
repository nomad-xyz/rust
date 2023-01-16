#[derive(Debug)]
pub enum TokenName {
    USDC,
    CQT,
    USDT,
    FRAX,
    WBTC,
    IAG,
    WETH,
    DAI,
    C3,
    FXS,
    CARDS,
    HBOT,
    SDL,
    GERO,
}

#[derive(Debug)]
pub struct Token {
    pub name: TokenName,
    pub id: String,
    pub decimals: f64,
    pub contract_address: String,
    pub recovered_total: f64,
}
