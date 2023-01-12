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

pub struct Token {
    pub name: TokenName,
    pub contract_address: String,
}
