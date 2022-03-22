use color_eyre::{eyre::bail, Report};
use ethers_signers::WalletError;
pub use nomad_types::NomadIdentifier;
use std::convert::Infallible;

use async_trait::async_trait;
use ethers::{
    core::types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address as EthAddress, Signature,
    },
    prelude::AwsSigner,
    signers::{AwsSignerError, LocalWallet, Signer},
};
use nomad_xyz_configuration::agent::SignerConf;
use once_cell::sync::OnceCell;
use rusoto_core::{credential::EnvironmentProvider, HttpClient};
use rusoto_kms::KmsClient;

static KMS_CLIENT: OnceCell<KmsClient> = OnceCell::new();

/// Error types for Signers
#[derive(Debug, thiserror::Error)]
pub enum SignersError {
    /// AWS Signer Error
    #[error("{0}")]
    AwsSignerError(#[from] AwsSignerError),
    /// Wallet Signer Error
    #[error("{0}")]
    WalletError(#[from] WalletError),
}

impl From<Infallible> for SignersError {
    fn from(_error: Infallible) -> Self {
        panic!("infallible")
    }
}

/// Ethereum-supported signer types
#[derive(Debug, Clone)]
pub enum Signers {
    /// A wallet instantiated with a locally stored private key
    Local(LocalWallet),
    /// A signer using a key stored in aws kms
    Aws(AwsSigner<'static>),
}

impl From<LocalWallet> for Signers {
    fn from(s: LocalWallet) -> Self {
        Signers::Local(s)
    }
}

impl From<AwsSigner<'static>> for Signers {
    fn from(s: AwsSigner<'static>) -> Self {
        Signers::Aws(s)
    }
}

impl Signers {
    /// Try to build Signer from SignerConf object
    pub async fn try_from_signer_conf(conf: &SignerConf) -> Result<Self, Report> {
        match conf {
            SignerConf::HexKey { key } => Ok(Self::Local(key.as_ref().parse()?)),
            SignerConf::Aws { id, region } => {
                let client = KMS_CLIENT.get_or_init(|| {
                    KmsClient::new_with_client(
                        rusoto_core::Client::new_with(
                            EnvironmentProvider::default(),
                            HttpClient::new().unwrap(),
                        ),
                        region.parse().expect("invalid region"),
                    )
                });

                let signer = AwsSigner::new(client, id, 0).await?;
                Ok(Self::Aws(signer))
            }
            SignerConf::Node => bail!("Node signer"),
        }
    }
}

#[async_trait]
impl Signer for Signers {
    type Error = SignersError;

    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        match self {
            Signers::Local(signer) => signer.with_chain_id(chain_id).into(),
            Signers::Aws(signer) => signer.with_chain_id(chain_id).into(),
        }
    }

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_message(message).await?),
            Signers::Aws(signer) => Ok(signer.sign_message(message).await?),
        }
    }

    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_transaction(message).await?),

            Signers::Aws(signer) => Ok(signer.sign_transaction(message).await?),
        }
    }

    fn address(&self) -> EthAddress {
        match self {
            Signers::Local(signer) => signer.address(),
            Signers::Aws(signer) => signer.address(),
        }
    }

    fn chain_id(&self) -> u64 {
        match self {
            Signers::Local(signer) => signer.chain_id(),
            Signers::Aws(signer) => signer.chain_id(),
        }
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        match self {
            Signers::Local(signer) => Ok(signer.sign_typed_data(payload).await?),
            Signers::Aws(signer) => Ok(signer.sign_typed_data(payload).await?),
        }
    }
}

/// Extension of ethers signer trait
#[async_trait]
pub trait SignerExt: Signer {
    /// Sign message without eip 155
    async fn sign_message_without_eip_155<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, <Self as Signer>::Error> {
        let mut signature = self.sign_message(message).await?;
        signature.v = 28 - (signature.v % 2);
        Ok(signature)
    }
}

impl<T> SignerExt for T where T: Signer {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Update;
    use ethers::types::H256;

    #[test]
    fn it_sign() {
        let t = async {
            let signer: ethers::signers::LocalWallet =
                "1111111111111111111111111111111111111111111111111111111111111111"
                    .parse()
                    .unwrap();
            let message = Update {
                home_domain: 5,
                new_root: H256::repeat_byte(1),
                previous_root: H256::repeat_byte(2),
            };

            let signed = message.sign_with(&signer).await.expect("!sign_with");
            assert!(signed.signature.v == 27 || signed.signature.v == 28);
            signed.verify(signer.address()).expect("!verify");
        };
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(t)
    }
}
