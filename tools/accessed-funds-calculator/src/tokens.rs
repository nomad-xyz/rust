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

impl Token {
    pub fn get_instance_of(token_name: TokenName) -> Token {
        match token_name {
            TokenName::Usdc => Token {
                name: TokenName::Usdc,
                id: "usd-coin".to_string(),
                decimals: 6.0,
                contract_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                recovered_total: 12_890_538.932_401,
            },
            TokenName::Cqt => Token {
                name: TokenName::Cqt,
                id: "covalent".to_string(),
                decimals: 18.0,
                contract_address: "0xD417144312DbF50465b1C641d016962017Ef6240".to_string(),
                recovered_total: 34_082_775.751_599_7,
            },
            TokenName::Usdt => Token {
                name: TokenName::Usdt,
                id: "tether".to_string(),
                decimals: 6.0,
                contract_address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
                recovered_total: 4_673_863.595_197,
            },
            TokenName::Frax => Token {
                name: TokenName::Frax,
                id: "frax".to_string(),
                decimals: 18.0,
                contract_address: "0x853d955aCEf822Db058eb8505911ED77F175b99e".to_string(),
                recovered_total: 2_644_469.918_609_09,
            },
            TokenName::Wbtc => Token {
                name: TokenName::Wbtc,
                id: "wrapped-bitcoin".to_string(),
                decimals: 8.0,
                contract_address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string(),
                recovered_total: 280.731_173_99,
            },
            TokenName::Iag => Token {
                name: TokenName::Iag,
                id: "iagon".to_string(),
                decimals: 18.0,
                contract_address: "0x40EB746DEE876aC1E78697b7Ca85142D178A1Fc8".to_string(),
                recovered_total: 349_507_392.187_402,
            },
            TokenName::Weth => Token {
                name: TokenName::Weth,
                id: "weth".to_string(),
                decimals: 18.0,
                contract_address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
                recovered_total: 1_049.635_629_8,
            },
            TokenName::Dai => Token {
                name: TokenName::Dai,
                id: "dai".to_string(),
                decimals: 18.0,
                contract_address: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
                recovered_total: 866_070.756_876_35,
            },
            TokenName::C3 => Token {
                name: TokenName::C3,
                id: "charli3".to_string(),
                decimals: 18.0,
                contract_address: "0xf1a91C7d44768070F711c68f33A7CA25c8D30268".to_string(),
                recovered_total: 1_684_711.122_391_36,
            },
            TokenName::Fxs => Token {
                name: TokenName::Fxs,
                id: "frax-share".to_string(),
                decimals: 18.0,
                contract_address: "0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0".to_string(),
                recovered_total: 46_895.688_044_5,
            },
            TokenName::Cards => Token {
                name: TokenName::Cards,
                id: "cardstarter".to_string(),
                decimals: 18.0,
                contract_address: "0x3d6F0DEa3AC3C607B3998e6Ce14b6350721752d9".to_string(),
                recovered_total: 165_005.819_480_28,
            },
            TokenName::Hbot => Token {
                name: TokenName::Hbot,
                id: "hummingbot".to_string(),
                decimals: 18.0,
                contract_address: "0xE5097D9baeAFB89f9bcB78C9290d545dB5f9e9CB".to_string(),
                recovered_total: 900_239.997_966,
            },
            TokenName::Sdl => Token {
                name: TokenName::Sdl,
                id: "saddle-finance".to_string(),
                decimals: 18.0,
                contract_address: "0xf1Dc500FdE233A4055e25e5BbF516372BC4F6871".to_string(),
                recovered_total: 9_790.824_057,
            },
            TokenName::Gero => Token {
                name: TokenName::Gero,
                id: "gerowallet".to_string(),
                decimals: 18.0,
                contract_address: "0x3431F91b3a388115F00C5Ba9FdB899851D005Fb5".to_string(),
                recovered_total: 23_245_641.666_183_1,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_usdc() {
        let usdc = Token {
            name: TokenName::Usdc,
            id: "usd-coin".to_string(),
            decimals: 6.0,
            contract_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            recovered_total: 12_890_538.932_401,
        };
        let test_token = Token::get_instance_of(TokenName::Usdc);
        assert_eq!(test_token.id, usdc.id);
        assert_eq!(test_token.decimals, usdc.decimals);
        assert_eq!(test_token.recovered_total, usdc.recovered_total);
    }
    #[test]
    fn validate_cqt() {
        let usdc = Token {
            name: TokenName::Cqt,
            id: "covalent".to_string(),
            decimals: 18.0,
            contract_address: "0xD417144312DbF50465b1C641d016962017Ef6240".to_string(),
            recovered_total: 34_082_775.751_599_7,
        };
        let test_token = Token::get_instance_of(TokenName::Cqt);
        assert_eq!(test_token.id, usdc.id);
        assert_eq!(test_token.decimals, usdc.decimals);
        assert_eq!(test_token.recovered_total, usdc.recovered_total);
    }
    #[test]
    fn validate_usdt() {
        let usdc = Token {
            name: TokenName::Usdt,
            id: "tether".to_string(),
            decimals: 6.0,
            contract_address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            recovered_total: 4_673_863.595_197,
        };
        let test_token = Token::get_instance_of(TokenName::Usdt);
        assert_eq!(test_token.id, usdc.id);
        assert_eq!(test_token.decimals, usdc.decimals);
        assert_eq!(test_token.recovered_total, usdc.recovered_total);
    }
    #[test]
    fn validate_frax() {
        let usdc = Token {
            name: TokenName::Frax,
            id: "frax".to_string(),
            decimals: 18.0,
            contract_address: "0x853d955aCEf822Db058eb8505911ED77F175b99e".to_string(),
            recovered_total: 2_644_469.918_609_09,
        };
        let test_token = Token::get_instance_of(TokenName::Frax);
        assert_eq!(test_token.id, usdc.id);
        assert_eq!(test_token.decimals, usdc.decimals);
        assert_eq!(test_token.recovered_total, usdc.recovered_total);
    }
    #[test]
    fn validate_wbtc() {
        let usdc = Token {
            name: TokenName::Wbtc,
            id: "wrapped-bitcoin".to_string(),
            decimals: 8.0,
            contract_address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string(),
            recovered_total: 280.731_173_99,
        };
        let test_token = Token::get_instance_of(TokenName::Wbtc);
        assert_eq!(test_token.id, usdc.id);
        assert_eq!(test_token.decimals, usdc.decimals);
        assert_eq!(test_token.recovered_total, usdc.recovered_total);
    }
    #[test]
    fn validate_iag() {
        let usdc = Token {
            name: TokenName::Iag,
            id: "iagon".to_string(),
            decimals: 18.0,
            contract_address: "0x40EB746DEE876aC1E78697b7Ca85142D178A1Fc8".to_string(),
            recovered_total: 349_507_392.187_402,
        };
        let test_token = Token::get_instance_of(TokenName::Iag);
        assert_eq!(test_token.id, usdc.id);
        assert_eq!(test_token.decimals, usdc.decimals);
        assert_eq!(test_token.recovered_total, usdc.recovered_total);
    }
    #[test]
    fn validate_weth() {
        let usdc = Token {
            name: TokenName::Weth,
            id: "weth".to_string(),
            decimals: 18.0,
            contract_address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            recovered_total: 1_049.635_629_8,
        };
        let test_token = Token::get_instance_of(TokenName::Weth);
        assert_eq!(test_token.id, usdc.id);
        assert_eq!(test_token.decimals, usdc.decimals);
        assert_eq!(test_token.recovered_total, usdc.recovered_total);
    }
    #[test]
    fn validate_dai() {
        let usdc = Token {
            name: TokenName::Dai,
            id: "dai".to_string(),
            decimals: 18.0,
            contract_address: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
            recovered_total: 866_070.756_876_35,
        };
        let test_token = Token::get_instance_of(TokenName::Dai);
        assert_eq!(test_token.id, usdc.id);
        assert_eq!(test_token.decimals, usdc.decimals);
        assert_eq!(test_token.recovered_total, usdc.recovered_total);
    }
    #[test]
    fn validate_c3() {
        let c3 = Token {
            name: TokenName::C3,
            id: "charli3".to_string(),
            decimals: 18.0,
            contract_address: "0xf1a91C7d44768070F711c68f33A7CA25c8D30268".to_string(),
            recovered_total: 1_684_711.122_391_36,
        };
        let test_token = Token::get_instance_of(TokenName::C3);
        assert_eq!(test_token.id, c3.id);
        assert_eq!(test_token.decimals, c3.decimals);
        assert_eq!(test_token.recovered_total, c3.recovered_total);
    }
    #[test]
    fn validate_fxs() {
        let fxs = Token {
            name: TokenName::Fxs,
            id: "frax-share".to_string(),
            decimals: 18.0,
            contract_address: "0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0".to_string(),
            recovered_total: 46_895.688_044_5,
        };
        let test_token = Token::get_instance_of(TokenName::Fxs);
        assert_eq!(test_token.id, fxs.id);
        assert_eq!(test_token.decimals, fxs.decimals);
        assert_eq!(test_token.recovered_total, fxs.recovered_total);
    }
    #[test]
    fn validate_cards() {
        let cards = Token {
            name: TokenName::Cards,
            id: "cardstarter".to_string(),
            decimals: 18.0,
            contract_address: "0x3d6F0DEa3AC3C607B3998e6Ce14b6350721752d9".to_string(),
            recovered_total: 165_005.819_480_28,
        };
        let test_token = Token::get_instance_of(TokenName::Cards);
        assert_eq!(test_token.id, cards.id);
        assert_eq!(test_token.decimals, cards.decimals);
        assert_eq!(test_token.recovered_total, cards.recovered_total);
    }
    #[test]
    fn validate_hbot() {
        let hbot = Token {
            name: TokenName::Hbot,
            id: "hummingbot".to_string(),
            decimals: 18.0,
            contract_address: "0xE5097D9baeAFB89f9bcB78C9290d545dB5f9e9CB".to_string(),
            recovered_total: 900_239.997_966,
        };
        let test_token = Token::get_instance_of(TokenName::Hbot);
        assert_eq!(test_token.id, hbot.id);
        assert_eq!(test_token.decimals, hbot.decimals);
        assert_eq!(test_token.recovered_total, hbot.recovered_total);
    }
    #[test]
    fn validate_sdl() {
        let sdl = Token {
            name: TokenName::Sdl,
            id: "saddle-finance".to_string(),
            decimals: 18.0,
            contract_address: "0xf1Dc500FdE233A4055e25e5BbF516372BC4F6871".to_string(),
            recovered_total: 9_790.824_057,
        };
        let test_token = Token::get_instance_of(TokenName::Sdl);
        assert_eq!(test_token.id, sdl.id);
        assert_eq!(test_token.decimals, sdl.decimals);
        assert_eq!(test_token.recovered_total, sdl.recovered_total);
    }
    #[test]
    fn validate_gero() {
        let gero = Token {
            name: TokenName::Gero,
            id: "gerowallet".to_string(),
            decimals: 18.0,
            contract_address: "0x3431F91b3a388115F00C5Ba9FdB899851D005Fb5".to_string(),
            recovered_total: 23_245_641.666_183_1,
        };
        let test_token = Token::get_instance_of(TokenName::Gero);
        assert_eq!(test_token.id, gero.id);
        assert_eq!(test_token.decimals, gero.decimals);
        assert_eq!(test_token.recovered_total, gero.recovered_total);
    }
}
