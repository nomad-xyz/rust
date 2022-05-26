use std::collections::HashMap;

use ethers::types::Address;
use lazy_static::lazy_static;
use std::str::FromStr;

lazy_static! {
    pub static ref CHAIN_ID_TO_FORWARDER: HashMap<usize, Address> = HashMap::from(
    // Kovan
    [(
        42,
        Address::from_str("0x4F36f93F58d36DcbC1E60b9bdBE213482285C482").expect("!forwarder proxy"),
    ),
    // Goerli
    (
        5,
        Address::from_str("0x61BF11e6641C289d4DA1D59dC3E03E15D2BA971c").expect("!forwarder proxy"),
    ),
    // Rinkeby
    (
        4,
        Address::from_str("0x9B79b798563e538cc326D03696B3Be38b971D282").expect("!forwarder proxy"),
    ),
    ]);
}
