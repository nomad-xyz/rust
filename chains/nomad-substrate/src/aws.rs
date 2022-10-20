use async_trait::async_trait;
use color_eyre::Result;
use ethers_core::k256::ecdsa::{Signature as EthersSignature, VerifyingKey as EthersVerifyingKey};
use ethers_signers::AwsSigner as EthersAwsSigner;
use nomad_core::aws::get_kms_client;
use std::time::Duration;
use subxt::{
    error::SecretStringError,
    ext::{
        sp_core::{
            crypto::{CryptoTypePublicPair, Derive, UncheckedFrom},
            ecdsa, ByteArray, DeriveJunction, Pair as TraitPair, Public as TraitPublic,
        },
        sp_runtime::{CryptoType, MultiSignature, MultiSigner},
    },
};
use tokio::time::sleep;

// TODO: Rename things

/// Error types for AWS `Pair`
#[derive(Debug, thiserror::Error)]
pub enum AwsPairError {
    /// Dummy error
    #[error("Dummy error")]
    DummyError(),
}

/// A partially implemented Pair (`subxt::ext::sp_core::Pair`) that
/// will support a remote AWS signer using ECDSA
#[derive(Clone)]
pub struct Pair {
    signer: EthersAwsSigner<'static>,
    pubkey: Public,
    max_retries: u32,
    min_retry_ms: u64,
}

impl Pair {
    /// Create a new AWS Pair from an AWS id
    pub async fn new<T>(id: T) -> Result<Self>
    where
        T: AsRef<str> + Send + Sync,
    {
        let kms_client = get_kms_client().await;
        let signer = EthersAwsSigner::new(kms_client, id, 0)
            .await
            .map_err(|_| AwsPairError::DummyError())?;
        let pubkey = signer
            .get_pubkey()
            .await
            .map_err(|_| AwsPairError::DummyError())?;
        let pubkey = pubkey.try_into().map_err(|_| AwsPairError::DummyError())?;
        let max_retries = 5;
        let min_retry_ms = 1000;
        Ok(Self {
            signer,
            pubkey,
            max_retries,
            min_retry_ms,
        })
    }

    /// Our `Public` key
    fn public_remote(&self) -> Public {
        self.pubkey
    }

    /// Try to sign `message` using our remote signer. Since we can't recover
    /// from an error here, we'll discard it in favor of an `Option`
    async fn try_sign_remote(&self, _message: &[u8], delay: Duration) -> Option<Signature> {
        sleep(delay).await;
        let dummy = [0u8; 32]; // TODO:
        self.signer
            .sign_digest(dummy)
            .await
            .ok()
            .map(Into::<Signature>::into)
    }

    /// Try to sign `message` `max_retries` times with an exponential backoff between tries.
    /// If we hit `max_retries` `panic` since we're unable to return an error here.
    fn sign_remote(&self, message: &[u8]) -> Signature {
        let mut times_attempted = 0;
        let mut delay = Duration::from_millis(0);
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("unable to create tokio::runtime (this should never happen)")
            .block_on(async {
                loop {
                    match self.try_sign_remote(message, delay).await {
                        Some(signature) => return signature,
                        None => {
                            delay = Duration::from_millis(self.min_retry_ms.pow({
                                times_attempted += 1;
                                times_attempted
                            }))
                        }
                    }
                    if times_attempted == self.max_retries {
                        panic!(
                            "giving up after attempting to sign message {} times",
                            times_attempted
                        )
                    }
                }
            })
    }
}

/// To make `Pair` init from an AWS id while keeping our internal signer
/// generic over all `subxt::ext::sp_core::Pair` and `subxt::Config`.
/// This will be implemented as a noop for `subxt::ext::sp_core::ecdsa::Pair`
/// and other core implementations
#[async_trait]
pub trait FromAwsId {
    /// Create an AWS-compatible signer from an AWS id
    async fn from_aws_id<T>(id: T) -> Result<Self>
    where
        T: AsRef<str> + Send + Sync,
        Self: Sized;
}

#[async_trait]
impl FromAwsId for Pair {
    async fn from_aws_id<T>(id: T) -> Result<Self>
    where
        T: AsRef<str> + Send + Sync,
    {
        Pair::new(id).await
    }
}

#[async_trait]
impl FromAwsId for ecdsa::Pair {
    async fn from_aws_id<T: AsRef<str>>(_id: T) -> Result<Self>
    where
        T: AsRef<str> + Send + Sync,
    {
        unimplemented!("For compatibility only, ecdsa::Pair cannot be created from an AWS id")
    }
}

/// A `Public` key that is compatible with `subxt::ext::sp_core::Pair`
/// and AWS's ECDSA KMS signer
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Public(pub [u8; 33]);

impl UncheckedFrom<[u8; 33]> for Public {
    fn unchecked_from(x: [u8; 33]) -> Self {
        Public(x)
    }
}

impl Derive for Public {}

impl CryptoType for Public {
    type Pair = Pair;
}

impl ByteArray for Public {
    const LEN: usize = 33;
}

impl std::fmt::Display for Public {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }
}

impl AsRef<[u8]> for Public {
    fn as_ref(&self) -> &[u8] {
        todo!()
    }
}

impl AsMut<[u8]> for Public {
    fn as_mut(&mut self) -> &mut [u8] {
        todo!()
    }
}

impl TryFrom<EthersVerifyingKey> for Public {
    type Error = ();

    fn try_from(data: EthersVerifyingKey) -> Result<Self, Self::Error> {
        let data = data.to_bytes();
        TryFrom::<&[u8]>::try_from(data.as_slice())
    }
}

impl TryFrom<&[u8]> for Public {
    type Error = ();

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() != Self::LEN {
            return Err(());
        }
        let mut r = [0u8; Self::LEN];
        r.copy_from_slice(data);
        Ok(Self::unchecked_from(r))
    }
}

impl TraitPublic for Public {
    fn to_public_crypto_pair(&self) -> CryptoTypePublicPair {
        todo!()
    }
}

impl From<Public> for MultiSigner {
    fn from(_x: Public) -> Self {
        todo!()
    }
}

/// A `Signature` that is compatible with `subxt::ext::sp_core::Pair`
/// and AWS's ECDSA KMS signer
#[derive(PartialEq, Eq, Hash)]
pub struct Signature(pub [u8; 65]);

impl From<EthersSignature> for Signature {
    fn from(_ethers_signature: EthersSignature) -> Self {
        todo!()
    }
}

impl From<Signature> for MultiSignature {
    fn from(_x: Signature) -> Self {
        todo!()
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

/// We only need this to satisfy the associated type on
/// `subxt::ext::sp_core::Pair`, so we'll make it a seed of length zero
type DummySeed = [u8; 0];

/// The trait `subxt::ext::sp_core::Pair` handles signing, verification and the creation
/// of keypairs form local key material (mnemonics, random bytes, etc.). With a remote
/// AWS signer keypair creation is handled externally so we will only partially implement
/// `Pair` to reflect this.
impl TraitPair for Pair {
    type Public = Public;
    type Seed = DummySeed;
    type Signature = Signature;
    type DeriveError = ();

    /// Our `Public` key
    fn public(&self) -> Self::Public {
        self.public_remote()
    }

    /// Sign a message of arbitrary bytes to return a `Signature`
    fn sign(&self, message: &[u8]) -> Self::Signature {
        self.sign_remote(message)
    }

    fn verify<M: AsRef<[u8]>>(_sig: &Self::Signature, _message: M, _pubkey: &Self::Public) -> bool {
        todo!()
    }

    fn verify_weak<P: AsRef<[u8]>, M: AsRef<[u8]>>(_sig: &[u8], _message: M, _pubkey: P) -> bool {
        todo!()
    }

    /// Not implemented for AWS Pair
    fn generate_with_phrase(_password: Option<&str>) -> (Self, String, Self::Seed) {
        unimplemented!("Pair cannot be created with local key material")
    }

    /// Not implemented for AWS Pair
    fn from_phrase(
        _phrase: &str,
        _password: Option<&str>,
    ) -> Result<(Self, Self::Seed), SecretStringError> {
        unimplemented!("Pair cannot be created with local key material")
    }

    /// Not implemented for AWS Pair
    fn derive<Iter: Iterator<Item = DeriveJunction>>(
        &self,
        _path: Iter,
        _seed: Option<Self::Seed>,
    ) -> Result<(Self, Option<Self::Seed>), Self::DeriveError> {
        unimplemented!("Pair does not support derivation")
    }

    /// Not implemented for AWS Pair
    fn from_seed(_seed: &Self::Seed) -> Self {
        unimplemented!("Pair cannot be created with local key material")
    }

    /// Not implemented for AWS Pair
    fn from_seed_slice(_seed: &[u8]) -> Result<Self, SecretStringError> {
        unimplemented!("Pair cannot be created with local key material")
    }

    /// Not implemented for AWS Pair
    fn to_raw_vec(&self) -> Vec<u8> {
        unimplemented!("Pair does not have access to key material")
    }
}

impl CryptoType for Pair {
    type Pair = Pair;
}
