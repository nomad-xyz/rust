use color_eyre::{eyre::bail, Result};
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

/// Error types for EthereumSigners
#[derive(Debug, thiserror::Error)]
pub enum EthereumSignersError {
    /// AWS Signer Error
    #[error("{0}")]
    AwsSignerError(#[from] AwsSignerError),
    /// Wallet Signer Error
    #[error("{0}")]
    WalletError(#[from] WalletError),
}

impl From<Infallible> for EthereumSignersError {
    fn from(_error: Infallible) -> Self {
        panic!("infallible")
    }
}

/// Ethereum-supported signer types
#[derive(Debug, Clone)]
pub enum EthereumSigners {
    /// A wallet instantiated with a locally stored private key
    Local(LocalWallet),
    /// A signer using a key stored in aws kms
    Aws(AwsSigner<'static>),
}

impl From<LocalWallet> for EthereumSigners {
    fn from(s: LocalWallet) -> Self {
        EthereumSigners::Local(s)
    }
}

impl From<AwsSigner<'static>> for EthereumSigners {
    fn from(s: AwsSigner<'static>) -> Self {
        EthereumSigners::Aws(s)
    }
}

impl EthereumSigners {
    /// Try to build Signer from SignerConf object
    pub async fn try_from_signer_conf(conf: &SignerConf) -> Result<Self> {
        match conf {
            SignerConf::HexKey(key) => Ok(Self::Local(key.as_ref().parse()?)),
            SignerConf::Aws { id } => {
                let kms_client = crate::aws::get_kms_client().await;
                let signer = AwsSigner::new(kms_client, id, 0).await?;
                Ok(Self::Aws(signer))
            }
            SignerConf::Node => bail!("Node signer"),
        }
    }
}

#[async_trait]
impl Signer for EthereumSigners {
    type Error = EthereumSignersError;

    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        match self {
            EthereumSigners::Local(signer) => signer.with_chain_id(chain_id).into(),
            EthereumSigners::Aws(signer) => signer.with_chain_id(chain_id).into(),
        }
    }

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        match self {
            EthereumSigners::Local(signer) => Ok(signer.sign_message(message).await?),
            EthereumSigners::Aws(signer) => Ok(signer.sign_message(message).await?),
        }
    }

    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        match self {
            EthereumSigners::Local(signer) => Ok(signer.sign_transaction(message).await?),

            EthereumSigners::Aws(signer) => Ok(signer.sign_transaction(message).await?),
        }
    }

    fn address(&self) -> EthAddress {
        match self {
            EthereumSigners::Local(signer) => signer.address(),
            EthereumSigners::Aws(signer) => signer.address(),
        }
    }

    fn chain_id(&self) -> u64 {
        match self {
            EthereumSigners::Local(signer) => signer.chain_id(),
            EthereumSigners::Aws(signer) => signer.chain_id(),
        }
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        match self {
            EthereumSigners::Local(signer) => Ok(signer.sign_typed_data(payload).await?),
            EthereumSigners::Aws(signer) => Ok(signer.sign_typed_data(payload).await?),
        }
    }
}

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
