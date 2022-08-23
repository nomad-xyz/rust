use async_trait::async_trait;
use ethers::prelude::{Signature, Signer};

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
