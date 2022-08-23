use async_trait::async_trait;
use color_eyre::Result;
use ethers::prelude::{Signature, Signer};
use nomad_xyz_configuration::agent::SignerConf;

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

/// Interface for instantiating a chain-specific signer from a `SignerConf`
/// object.
#[async_trait]
pub trait FromSignerConf: Sized {
    /// Instantiate `Self` from a `SignerConf` object
    async fn try_from_signer_conf(conf: &SignerConf) -> Result<Self>;
}
