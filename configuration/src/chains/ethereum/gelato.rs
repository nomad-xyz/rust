use crate::{agent::SignerConf, FromEnv};

/// Configuration for tx submission through Gelato relay
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoConf {
    /// Sponsor signer configuration
    pub signer: SignerConf,
    /// Address of fee token
    pub fee_token: String,
}

/*
   {
       "transactionSubmission": {
           "ethereum": {
               "type": "gelato",
               "sponsorSigner": {
                   ...
               },
               "feeToken": "0xabc"
           },
           "moonbeam": {
               "type": "local",
               "sponsorSigner": {
                   ...
               },
           }
       }
   }


   TXSUBMISSION_KOVAN_TYPE=local
   TRANSACTIONSIGNERS_KOVAN_TYPE=hexKey
   TRANSACTIONSIGNERS_KOVAN_KEY=0x0000001111111111111111111111111111111111111111111111111111ce1002

   TXSUBMISSION_KOVAN_TYPE=gelato
   SPONSORSIGNER_KOVAN_TYPE=hexKey
   SPONSORSIGNER_KOVAN_KEY=0x0000001111111111111111111111111111111111111111111111111111ce1002

   TXSUBMISSION_KOVAN_TYPE=gelato
   SPONSORSIGNER_KOVAN_TYPE=aws
   SPONSORSIGNER_KOVAN_ID=...
   SPONSORSIGNER_KOVAN_REGION=...
*/

impl FromEnv for GelatoConf {
    fn from_env(prefix: &str) -> Option<Self> {
        let signer = SignerConf::from_env(&format!("{}_SIGNER", prefix))?;
        let fee_token = std::env::var(&format!("{}_FEETOKEN", prefix)).ok()?;

        Some(Self { signer, fee_token })
    }
}
