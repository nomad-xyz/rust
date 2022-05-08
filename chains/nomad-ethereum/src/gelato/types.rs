use ethers::abi::{self, Token};
use ethers::types::{transaction::eip712::*, Address};
use ethers::utils::hex::FromHexError;
use ethers::utils::keccak256;
use gelato_relay::ForwardRequest;
use std::str::FromStr;

const FORWARD_REQUEST_TYPE: &str = "ForwardRequest(uint256 chainId,address target,bytes data,address feeToken,uint256 paymentType,uint256 maxFee,address sponsor,uint256 sponsorChainId,uint256 nonce,bool enforceSponsorNonce)";

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct UnfilledFowardRequest {
    pub type_id: String,
    pub chain_id: usize,
    pub target: String,
    pub data: String,
    pub fee_token: String,
    pub payment_type: usize, // 1 = gas tank
    pub max_fee: usize,
    pub sponsor: String,
    pub sponsor_chain_id: usize,     // same as chain_id
    pub nonce: usize,                // can default 0 if next field false
    pub enforce_sponsor_nonce: bool, // default false given replay safe
}

/// ForwardRequest error
#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum ForwardRequestError {
    /// Hex decoding error
    #[error("Hex decoding error: {0}")]
    FromHexError(#[from] FromHexError),
}

impl Eip712 for UnfilledFowardRequest {
    type Error = ForwardRequestError;

    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        Ok(EIP712Domain {
            name: "GelatoRelayForwarder".to_owned(),
            version: "V1".to_owned(),
            chain_id: self.chain_id.into(),
            verifying_contract: Address::from_str("0xC176f63f3827afE6789FD737f4679B299F97d108")
                .expect("!verifying contract"), // TODO: fetch from Gelato API
            salt: None,
        })
    }

    fn type_hash() -> Result<[u8; 32], Self::Error> {
        Ok(keccak256(FORWARD_REQUEST_TYPE))
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let encoded_request = abi::encode(&[
            Token::FixedBytes(Self::type_hash()?.to_vec()),
            Token::Uint(self.chain_id.into()),
            Token::Address(Address::from_str(&self.target).expect("!target")),
            Token::FixedBytes(keccak256(hex::decode(&self.data)?).to_vec()),
            Token::Address(Address::from_str(&self.fee_token).expect("!fee token")),
            Token::Uint(self.payment_type.into()),
            Token::Uint(self.max_fee.into()),
            Token::Address(Address::from_str(&self.sponsor).expect("!sponsor")),
            Token::Uint(self.sponsor_chain_id.into()),
            Token::Uint(self.nonce.into()),
            Token::Bool(self.enforce_sponsor_nonce),
        ]);

        Ok(keccak256(encoded_request))
    }
}

impl UnfilledFowardRequest {
    /// Fill ForwardRequest with sponsor signature and return full request struct
    pub fn into_filled(self, sponsor_signature: Vec<u8>) -> ForwardRequest {
        let hex_sig = format!("0x{}", hex::encode(sponsor_signature));
        let hex_data = format!("0x{}", self.data);

        ForwardRequest {
            type_id: self.type_id,
            chain_id: self.chain_id,
            target: self.target,
            data: hex_data,
            fee_token: self.fee_token,
            payment_type: self.payment_type,
            max_fee: self.max_fee.to_string(),
            sponsor: self.sponsor,
            sponsor_chain_id: self.sponsor_chain_id,
            nonce: self.nonce,
            enforce_sponsor_nonce: self.enforce_sponsor_nonce,
            sponsor_signature: hex_sig,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::UnfilledFowardRequest;
    use ethers::signers::LocalWallet;
    use ethers::signers::Signer;
    use ethers::types::transaction::eip712::Eip712;
    use lazy_static::lazy_static;

    const DUMMY_SPONSOR_KEY: &str =
        "fae558d7fb0ac7970a7a472559f332c2a67d2ec283c98fd2afa58403bdfd74a5";
    const SPONSOR_SIGNATURE: &str = "0xc09004502ade171bc918fbb6eb4911045b7defdb78435b37de693a9f4ee80d9e2b17d45237d1012e7df27aaac6a2b51072ba1fecfa4a151d1f85fbc278e85e7f1b";

    lazy_static! {
        pub static ref REQUEST: UnfilledFowardRequest = UnfilledFowardRequest {
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
    }

    #[test]
    fn it_computes_domain_separator() {
        let domain_separator = REQUEST.domain_separator().unwrap();

        assert_eq!(
            format!("0x{}", hex::encode(domain_separator)),
            "0x80d0833d2a99df6a94d491cee0d9b3b5586c41d9b01edaf54538f65d01474c94"
        );
    }

    #[tokio::test]
    async fn it_computes_and_signs_digest() {
        let sponsor: LocalWallet = DUMMY_SPONSOR_KEY.parse().unwrap();

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

        let signature = sponsor.sign_typed_data(&request).await.unwrap().to_vec();

        let hex_sig = format!("0x{}", hex::encode(signature));
        assert_eq!(SPONSOR_SIGNATURE, hex_sig);
    }
}
