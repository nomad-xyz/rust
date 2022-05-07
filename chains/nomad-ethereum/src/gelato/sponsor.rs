use color_eyre::Result;
use ethers::abi::{ethereum_types::BigEndianHash, Token};
use ethers::prelude::*;
use ethers::utils::keccak256;
use gelato_relay::ForwardRequest;
use lazy_static::lazy_static;
use nomad_core::SignerExt;
use sha3::{Digest, Keccak256};
use std::str::FromStr;

const EIP712_DOMAIN_TYPE: &str =
    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";

lazy_static! {
    /// Typehash of ForwardRequest signature
    pub static ref FORWARD_REQUEST_TYPEHASH: [u8; 32] = keccak256(
        &keccak256("ForwardRequest(uint256 chainId,address target,bytes data,address feeToken,uint256 paymentType,uint256 maxFee,address sponsor,uint256 sponsorChainId,uint256 nonce,bool enforceSponsorNonce)")
    );
}

/// Get domain separator for GelatoRelayerForwarder address and chain id
pub fn get_domain_separator(address: Address, chain_id: U256) -> H256 {
    let domain_separator = abi::encode(&[
        Token::FixedBytes(keccak256(EIP712_DOMAIN_TYPE).to_vec()),
        Token::FixedBytes(keccak256("GelatoRelayForwarder").to_vec()),
        Token::FixedBytes(keccak256("V1").to_vec()),
        Token::FixedBytes(H256::from_uint(&chain_id).as_bytes().to_vec()),
        Token::Address(address),
    ]);

    H256::from_slice(&keccak256(domain_separator))
}

/// Get hash of abi encoded ForwardRequest
pub fn get_forward_request_hash(request: &ForwardRequest) -> H256 {
    let encoded_request = abi::encode(&[
        Token::FixedBytes((*FORWARD_REQUEST_TYPEHASH).to_vec()),
        Token::Uint(request.chain_id.into()),
        Token::Address(Address::from_str(&request.target).expect("!target")),
        Token::FixedBytes(keccak256(&request.data).to_vec()),
        Token::Address(Address::from_str(&request.fee_token).expect("!feetoken")),
        Token::Uint(request.payment_type.into()),
        Token::Uint(U256::from_str(&request.max_fee).expect("!maxfee")),
        Token::Address(Address::from_str(&request.sponsor).expect("!sponsor")),
        Token::Uint(request.sponsor_chain_id.into()),
        Token::Uint(request.nonce.into()),
        Token::Bool(request.enforce_sponsor_nonce),
    ]);

    H256::from_slice(&encoded_request)
}

/// Sign request that will be given to GelatoRelayForwarder on given chain
pub async fn sponsor_sign_request<S: Signer>(
    sponsor: &S,
    forwarder: Address,
    request: &ForwardRequest,
) -> Result<Vec<u8>, S::Error> {
    assert!(
        request.sponsor_signature.is_none(),
        "Tried to sign already signed forward request: {:?}",
        request
    );

    let domain_separator = get_domain_separator(forwarder, request.chain_id.into());
    let request_hash = get_forward_request_hash(request);

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

#[cfg(test)]
mod test {
    use super::*;
    use ethers::prelude::U256;

    const KOVAN_GELATO_RELAY_FORWARDER: &str = "0xC176f63f3827afE6789FD737f4679B299F97d108";
    const KOVAN_CHAIN_ID: u64 = 42;

    #[test]
    fn it_computes_domain_separator() {
        let domain_separator = get_domain_separator(
            H160::from_str(KOVAN_GELATO_RELAY_FORWARDER).unwrap(),
            U256::from(KOVAN_CHAIN_ID),
        );
        dbg!("{:?}", domain_separator);

        assert_eq!(
            format!("{:#x}", domain_separator),
            "0x80d0833d2a99df6a94d491cee0d9b3b5586c41d9b01edaf54538f65d01474c94"
        );
    }
}
