use async_trait::async_trait;
use color_eyre::{eyre::bail, Result};
use nomad_core::FromSignerConf;
use nomad_xyz_configuration::agent::SignerConf;
use subxt::{
    ext::sp_core::Pair,
    ext::sp_runtime::traits::{IdentifyAccount, Verify},
    tx::{PairSigner, Signer},
    Config,
};

/// Substrate signer variants
pub enum SubstrateSigners<T: Config, P: Pair> {
    /// Local signer, instantiated from local private key
    Local(PairSigner<T, P>),
}

#[async_trait]
impl<T, P> FromSignerConf for SubstrateSigners<T, P>
where
    T: Config,
    T::Signature: From<P::Signature>,
    <T::Signature as Verify>::Signer: From<P::Public> + IdentifyAccount<AccountId = T::AccountId>,
    P: Pair,
{
    async fn try_from_signer_conf(conf: &SignerConf) -> Result<Self> {
        match conf {
            SignerConf::HexKey(key) => {
                let pair = P::from_string(key.as_ref(), None).unwrap(); // TODO: remove unwrap
                Ok(Self::Local(PairSigner::<T, _>::new(pair)))
            }
            SignerConf::Aws { .. } => bail!("No AWS signer support"),
            SignerConf::Node => bail!("No node signer support"),
        }
    }
}

impl<T: Config, P: Pair> Signer<T> for SubstrateSigners<T, P>
where
    T: Config,
    T::Signature: From<P::Signature>,
    <T::Signature as Verify>::Signer: From<P::Public> + IdentifyAccount<AccountId = T::AccountId>,
    T::AccountId: Into<T::Address> + Clone + 'static,
    P::Signature: Into<T::Signature> + 'static,
    P: Pair + 'static,
{
    fn nonce(&self) -> Option<<T as Config>::Index> {
        match self {
            SubstrateSigners::Local(signer) => signer.nonce(),
        }
    }

    fn account_id(&self) -> &<T as Config>::AccountId {
        match self {
            SubstrateSigners::Local(signer) => signer.account_id(),
        }
    }

    fn address(&self) -> <T as Config>::Address {
        match self {
            SubstrateSigners::Local(signer) => signer.address(),
        }
    }

    fn sign(&self, signer_payload: &[u8]) -> <T as Config>::Signature {
        match self {
            SubstrateSigners::Local(signer) => signer.sign(signer_payload),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::AvailConfig;
    use subxt::ext::sp_core::ecdsa;

    use super::*;

    #[tokio::test]
    async fn it_instantiates_and_signs() {
        let conf = SignerConf::HexKey(
            "1111111111111111111111111111111111111111111111111111111111111111"
                .parse()
                .unwrap(),
        );
        let signer = SubstrateSigners::<AvailConfig, ecdsa::Pair>::try_from_signer_conf(&conf)
            .await
            .unwrap();
        let _sig = signer.sign("message".as_bytes());
    }
}
