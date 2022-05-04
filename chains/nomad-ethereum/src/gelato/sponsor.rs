use ethers::prelude::*;
use lazy_static::lazy_static;
use sha3::{Digest, Keccak256};

pub(crate) const EIP712_DOMAIN_TYPE: &str =
    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";

lazy_static! {
    pub(crate) static ref FORWARD_REQUEST_TYPEHASH: H256 = H256::from_slice(
        Keccak256::new()
            .chain("ForwardRequest(uint256 chainId,address target,bytes data,address feeToken,uint256 paymentType,uint256 maxFee,address sponsor,uint256 sponsorChainId,uint256 nonce,bool enforceSponsorNonce)".as_bytes())
            .finalize()
            .as_slice(),
    );
}

pub(crate) fn get_domain_separator(address: Address, chain_id: u64) -> H256 {
    H256::from_slice(
        Keccak256::new()
            .chain(EIP712_DOMAIN_TYPE.as_bytes())
            .chain("GelatoRelayForwarder".as_bytes())
            .chain("V1".as_bytes())
            .chain(chain_id.to_be_bytes())
            .chain(address)
            .finalize()
            .as_slice(),
    )
}

// TODO: implement
// pub(crate) fn abi_encode_forward_request(...) -> H256 {}
