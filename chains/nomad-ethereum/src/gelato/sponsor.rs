use color_eyre::Result;
use ethers::abi::Token;
use ethers::prelude::*;
use ethers::utils::{hash_message, keccak256};
use gelato_relay::ForwardRequest;
use lazy_static::lazy_static;
use nomad_core::SignerExt;
use nomad_core::Signers;
use sha3::{Digest, Keccak256};
use std::str::FromStr;

pub(crate) const EIP712_DOMAIN_TYPE: &str =
    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";

lazy_static! {
    pub(crate) static ref FORWARD_REQUEST_TYPEHASH: [u8; 32] = keccak256(
        &keccak256("ForwardRequest(uint256 chainId,address target,bytes data,address feeToken,uint256 paymentType,uint256 maxFee,address sponsor,uint256 sponsorChainId,uint256 nonce,bool enforceSponsorNonce)")
    );
}

pub(crate) fn get_domain_separator(address: Address, chain_id: U256) -> H256 {
    let domain_separator = abi::encode(&[
        Token::FixedBytes(keccak256(EIP712_DOMAIN_TYPE).to_vec()),
        Token::FixedBytes(keccak256("GelatoRelayForwarder").to_vec()),
        Token::FixedBytes(keccak256("V1").to_vec()),
        Token::FixedBytes(format!("{:x}", chain_id).into_bytes()),
        Token::Address(address),
    ]);

    H256::from_slice(&keccak256(domain_separator))
}

pub(crate) fn forward_request_hash(request: ForwardRequest) -> H256 {
    let encoded_request = abi::encode(&[
        Token::FixedBytes((*FORWARD_REQUEST_TYPEHASH).to_vec()),
        Token::Uint(U256::from(request.chain_id)),
        Token::Address(Address::from_str(&request.target).expect("!target")),
        Token::FixedBytes(keccak256(&request.data).to_vec()),
        Token::Address(Address::from_str(&request.fee_token).expect("!feetoken")),
        Token::Uint(U256::from(request.payment_type)),
        Token::Uint(U256::from_str(&request.max_fee).expect("!maxfee")),
        Token::Address(Address::from_str(&request.sponsor).expect("!sponsor")),
        Token::Uint(U256::from(request.sponsor_chain_id)),
        Token::Uint(U256::from(request.nonce)),
        Token::Bool(request.enforce_sponsor_nonce),
    ]);

    H256::from_slice(&encoded_request)
}

pub(crate) async fn sponsor_sign<S: Signer>(
    sponsor: &S,
    domain_separator: H256,
    request_hash: H256,
) -> Result<Vec<u8>, S::Error> {
    let digest = H256::from_slice(
        Keccak256::new()
            .chain("\x19\x01")
            .chain(domain_separator)
            .chain(request_hash)
            .finalize()
            .as_slice(),
    );

    sponsor
        .sign_message_without_eip_155(digest)
        .await
        .map(|s| s.to_vec())
}
