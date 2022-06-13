use std::collections::HashMap;

use ethers::types::Address;
use once_cell::sync::Lazy;
use std::str::FromStr;

pub static CHAIN_ID_TO_FORWARDER: Lazy<HashMap<usize, Address>> = Lazy::new(|| {
    HashMap::from([
        // Kovan
        (
            42,
            Address::from_str("0x4F36f93F58d36DcbC1E60b9bdBE213482285C482")
                .expect("!forwarder proxy"),
        ),
        // Goerli
        (
            5,
            Address::from_str("0x61BF11e6641C289d4DA1D59dC3E03E15D2BA971c")
                .expect("!forwarder proxy"),
        ),
        // Rinkeby
        (
            4,
            Address::from_str("0x9B79b798563e538cc326D03696B3Be38b971D282")
                .expect("!forwarder proxy"),
        ),
        // Evmos
        (
            9001,
            Address::from_str("0x9561aCdf04C2B639dFfeCB357438e7B3eD979C5C")
                .expect("!forwarder proxy"),
        ),
        // BSC
        (
            56,
            Address::from_str("0xeeea839E2435873adA11d5dD4CAE6032742C0445")
                .expect("!forwarder proxy"),
        ),
        // Polygon
        (
            137,
            Address::from_str("0xc2336e796F77E4E57b6630b6dEdb01f5EE82383e")
                .expect("!forwarder proxy"),
        ),
    ])
});
