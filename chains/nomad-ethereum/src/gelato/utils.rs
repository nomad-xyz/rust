use std::collections::HashMap;

use ethers::types::Address;
use lazy_static::lazy_static;
use std::str::FromStr;

lazy_static! {
    pub static ref CHAIN_ID_TO_FORWARDER: HashMap<usize, Address> = HashMap::from(
    // Kovan
    [(
        42,
        Address::from_str("0xC176f63f3827afE6789FD737f4679B299F97d108").expect("!forwarder proxy"),
    ),
    // Goerli
    (
        5,
        Address::from_str("0xDde7416baE4CcfB1f131038482D424AdD61cF378").expect("!forwarder proxy"),
    ),
    // Rinkeby
    (
        4,
        Address::from_str("0x0343Af039E2E1c25A9691eEb654Ce0de1910C3e2").expect("!forwarder proxy"),
    ),
    ]);
}
