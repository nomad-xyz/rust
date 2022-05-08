use crate::UnfilledFowardRequest;
use color_eyre::Result;
use ethers::abi::{ethereum_types::BigEndianHash, Token};
use ethers::prelude::*;
use ethers::utils::keccak256;
use lazy_static::lazy_static;
use nomad_core::SignerExt;
use sha3::{Digest, Keccak256};
use std::str::FromStr;

const EIP712_DOMAIN_TYPE: &str =
    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";

lazy_static! {
    /// Typehash of ForwardRequest signature
    pub static ref FORWARD_REQUEST_TYPEHASH: [u8; 32] = keccak256("ForwardRequest(uint256 chainId,address target,bytes data,address feeToken,uint256 paymentType,uint256 maxFee,address sponsor,uint256 sponsorChainId,uint256 nonce,bool enforceSponsorNonce)");
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
pub fn get_forward_request_hash(request: &UnfilledFowardRequest) -> H256 {
    let encoded_request = abi::encode(&[
        Token::FixedBytes(FORWARD_REQUEST_TYPEHASH.to_vec()),
        Token::Uint(request.chain_id.into()),
        Token::Address(Address::from_str(&request.target).expect("!target")),
        Token::FixedBytes(keccak256(hex::decode(&request.data).expect("!data")).to_vec()),
        Token::Address(Address::from_str(&request.fee_token).expect("!feetoken")),
        Token::Uint(request.payment_type.into()),
        Token::Uint(request.max_fee.into()),
        Token::Address(Address::from_str(&request.sponsor).expect("!sponsor")),
        Token::Uint(request.sponsor_chain_id.into()),
        Token::Uint(request.nonce.into()),
        Token::Bool(request.enforce_sponsor_nonce),
    ]);

    println!(
        "Encoded request: {}",
        format!("0x{}", hex::encode(&encoded_request))
    );

    H256::from_slice(&keccak256(encoded_request))
}

/// Sign request that will be given to GelatoRelayForwarder on given chain
pub async fn sponsor_sign_request<S: Signer>(
    sponsor: &S,
    forwarder: Address,
    request: &UnfilledFowardRequest,
) -> Result<Vec<u8>, S::Error> {
    let domain_separator = get_domain_separator(forwarder, request.chain_id.into());
    let request_hash = get_forward_request_hash(request);
    println!("Forward request hash: {:?}", request_hash);

    // let digest = H256::from_slice(&keccak256(format!("0x{}{}{}",
    //     "1901",
    //     &format!("{:#x}", domain_separator)[2..],
    //     &format!("{:#x}", request_hash)[2..],
    // )));

    let digest = H256::from_slice(
        Keccak256::new()
            .chain("\x19\x01")
            .chain(domain_separator)
            .chain(request_hash)
            .finalize()
            .as_slice(),
    );

    println!("Digest: {}", format!("{:#x}", digest));

    sponsor
        .sign_message_without_eip_155(digest)
        .await
        .map(|s| s.to_vec())
}

#[cfg(test)]
mod test {
    use super::*;
    use ethers::signers::LocalWallet;

    const DUMMY_SPONSOR: &str = "fae558d7fb0ac7970a7a472559f332c2a67d2ec283c98fd2afa58403bdfd74a5";
    const KOVAN_GELATO_RELAY_FORWARDER: &str = "0xC176f63f3827afE6789FD737f4679B299F97d108";
    const KOVAN_CHAIN_ID: u64 = 42;
    const SPONSOR_SIGNATURE: &str = "0xc09004502ade171bc918fbb6eb4911045b7defdb78435b37de693a9f4ee80d9e2b17d45237d1012e7df27aaac6a2b51072ba1fecfa4a151d1f85fbc278e85e7f1b";

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

    #[tokio::test]
    async fn it_computes_and_signs_digest() {
        let sponsor: LocalWallet = DUMMY_SPONSOR.parse().unwrap();
        println!("Sponsor address: {:#x}", sponsor.address());

        let request = UnfilledFowardRequest {
            type_id: "ForwardRequest".to_owned(),
            chain_id: 42,
            target: "0x61bBe925A5D646cE074369A6335e5095Ea7abB7A".to_owned(),
            data: "4b327067000000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .to_owned(),
            fee_token: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE".to_owned(),
            payment_type: 1,
            max_fee: 10000000000000000000,
            sponsor: "0xcaCE8809B0F21A2dd707CA7B3E4CB04ffcCB5A3e".to_owned(),
            sponsor_chain_id: 42,
            nonce: 0,
            enforce_sponsor_nonce: false,
        };

        let signature = sponsor_sign_request(
            &sponsor,
            H160::from_str(KOVAN_GELATO_RELAY_FORWARDER).unwrap(),
            &request,
        )
        .await
        .unwrap();

        let hex_sig = format!("Signature: 0x{}", hex::encode(signature));
        println!("{}", hex_sig);
    }
}
