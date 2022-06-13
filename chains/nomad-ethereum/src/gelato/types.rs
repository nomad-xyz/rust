use super::utils::get_forwarder;
use ethers::abi::{self, Token};
use ethers::prelude::{Bytes, H160, U64};
use ethers::types::transaction::eip712::*;
use ethers::utils::hex::FromHexError;
use ethers::utils::keccak256;
use gelato_sdk::{ForwardRequest, PaymentType};

const FORWARD_REQUEST_TYPE: &str = "ForwardRequest(uint256 chainId,address target,bytes data,address feeToken,uint256 paymentType,uint256 maxFee,uint256 gas,address sponsor,uint256 sponsorChainId,uint256 nonce,bool enforceSponsorNonce,bool enforceSponsorNonceOrdering)";

/// Unfilled Gelato forward request. This request is signed and filled according
/// to EIP-712 then sent to Gelato. Gelato executes the provided tx `data` on
/// the `target` contract address.
#[derive(Debug, Clone)]
pub struct UnfilledForwardRequest {
    /// Target chain id
    pub chain_id: usize,
    /// Target contract address
    pub target: H160,
    /// Encoded tx data
    pub data: Bytes,
    /// Fee token address
    pub fee_token: H160,
    /// Payment method
    pub payment_type: PaymentType, // 1 = gas tank
    /// Max fee
    pub max_fee: U64,
    /// Contract call gas limit + buffer for gelato forwarder
    pub gas: U64,
    /// Sponsor address
    pub sponsor: H160,
    /// Sponsor resident chain id
    pub sponsor_chain_id: usize, // same as chain_id
    /// Nonce for replay protection
    pub nonce: usize, // can default 0 if next field false
    /// Enforce nonce replay protection
    pub enforce_sponsor_nonce: bool, // default false given replay safe
    /// Enforce ordering based on provided nonces. Only considered if
    /// `enforce_sponsor_nonce` true.
    pub enforce_sponsor_nonce_ordering: Option<bool>,
}

/// ForwardRequest error
#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum ForwardRequestError {
    /// Hex decoding error
    #[error("Hex decoding error: {0}")]
    FromHexError(#[from] FromHexError),
    /// Unknown forwarder
    #[error("Forwarder contract unknown for domain: {0}")]
    UnknownForwarderError(usize),
}

impl Eip712 for UnfilledForwardRequest {
    type Error = ForwardRequestError;

    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        let verifying_contract = get_forwarder(self.chain_id)
            .ok_or_else(|| ForwardRequestError::UnknownForwarderError(self.chain_id))?;

        Ok(EIP712Domain {
            name: "GelatoRelayForwarder".to_owned(),
            version: "V1".to_owned(),
            chain_id: self.chain_id.into(),
            verifying_contract,
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
            Token::Address(self.target),
            Token::FixedBytes(keccak256(&self.data).to_vec()),
            Token::Address(self.fee_token),
            Token::Uint((self.payment_type as u8).into()),
            Token::Uint(self.max_fee.as_u64().into()),
            Token::Uint(self.gas.as_u64().into()),
            Token::Address(self.sponsor),
            Token::Uint(self.sponsor_chain_id.into()),
            Token::Uint(self.nonce.into()),
            Token::Bool(self.enforce_sponsor_nonce),
            Token::Bool(self.enforce_sponsor_nonce_ordering.unwrap_or(true)),
        ]);

        Ok(keccak256(encoded_request))
    }
}

impl UnfilledForwardRequest {
    /// Fill ForwardRequest with sponsor signature and return full request struct
    pub fn into_filled(self, sponsor_signature: ethers::core::types::Signature) -> ForwardRequest {
        ForwardRequest {
            type_id: "ForwardRequest",
            chain_id: self.chain_id,
            target: self.target,
            data: self.data,
            fee_token: self.fee_token,
            payment_type: self.payment_type,
            max_fee: self.max_fee,
            gas: self.gas,
            sponsor: self.sponsor,
            sponsor_chain_id: self.sponsor_chain_id,
            nonce: self.nonce,
            enforce_sponsor_nonce: self.enforce_sponsor_nonce,
            enforce_sponsor_nonce_ordering: self.enforce_sponsor_nonce_ordering,
            sponsor_signature,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::UnfilledForwardRequest;
    use ethers::signers::LocalWallet;
    use ethers::signers::Signer;
    use ethers::types::transaction::eip712::Eip712;
    use once_cell::sync::Lazy;

    const DOMAIN_SEPARATOR: &str =
        "0x1b927f522830945610cf8f0521ef8b3f69352936e1b0920968dcad9cf1e30762";
    const DUMMY_SPONSOR_KEY: &str =
        "9cb3a530d61728e337290409d967db069f5219279f89e5ddb5ae4af76a8da5f4";
    const DUMMY_SPONSOR_ADDRESS: &str = "0x4e4f0d95bc1a4275b748a63221796080b1aa5c10";
    const SPONSOR_SIGNATURE: &str = "0x23c272c0cba2b897de0fd8fe87d419f0f273c82ef10917520b733da889688b1c6fec89412c6f121fccbc30ce89b20a3de2f405018f1ac1249b9ff705fdb62a521b";

    static REQUEST: Lazy<UnfilledForwardRequest> = Lazy::new(|| UnfilledForwardRequest {
        chain_id: 42,
        target: "0x61bBe925A5D646cE074369A6335e5095Ea7abB7A"
            .parse()
            .unwrap(),
        data: "4b327067000000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeaeeeeeeeeeeeeeeeee"
            .parse()
            .unwrap(),
        fee_token: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"
            .parse()
            .unwrap(),
        payment_type: gelato_sdk::PaymentType::AsyncGasTank,
        max_fee: 10000000000000000000u64.into(),
        gas: 200000u64.into(),
        sponsor: DUMMY_SPONSOR_ADDRESS.parse().unwrap(),
        sponsor_chain_id: 42,
        nonce: 0,
        enforce_sponsor_nonce: false,
        enforce_sponsor_nonce_ordering: Some(false),
    });

    #[test]
    fn it_computes_domain_separator() {
        let domain_separator = REQUEST.domain_separator().unwrap();

        assert_eq!(
            format!("0x{}", hex::encode(domain_separator)),
            DOMAIN_SEPARATOR,
        );
    }

    #[tokio::test]
    async fn it_computes_and_signs_digest() {
        let sponsor: LocalWallet = DUMMY_SPONSOR_KEY.parse().unwrap();
        assert_eq!(DUMMY_SPONSOR_ADDRESS, format!("{:#x}", sponsor.address()));

        let signature = sponsor.sign_typed_data(&*REQUEST).await.unwrap().to_vec();

        let hex_sig = format!("0x{}", hex::encode(signature));
        assert_eq!(SPONSOR_SIGNATURE, hex_sig);
    }
}
