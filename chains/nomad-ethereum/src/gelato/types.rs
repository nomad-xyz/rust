use gelato_relay::ForwardRequest;

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

impl UnfilledFowardRequest {
    /// Fill ForwardRequest with sponsor signature and return full request struct
    pub fn into_filled(self, sponsor_signature: Vec<u8>) -> ForwardRequest {
        let hex_sig = format!("0x{}", hex::encode(sponsor_signature));

        ForwardRequest {
            type_id: self.type_id,
            chain_id: self.chain_id,
            target: self.target,
            data: self.data,
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
