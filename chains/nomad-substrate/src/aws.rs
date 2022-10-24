use async_trait::async_trait;
use color_eyre::Result;
use ethers_core::{
    k256::ecdsa::VerifyingKey as EthersVerifyingKey, types::Signature as EthersSignature,
};
use ethers_signers::{AwsSigner as EthersAwsSigner, AwsSignerError, Signer};
use nomad_core::aws::get_kms_client;
use std::time::Duration;
use subxt::{
    error::SecretStringError,
    ext::{
        sp_core::{
            crypto::{CryptoTypePublicPair, Derive, UncheckedFrom},
            ecdsa, ByteArray, DeriveJunction, Pair as TraitPair, Public as TraitPublic,
        },
        sp_runtime::{CryptoType, MultiSignature},
    },
};
use tokio::time::sleep;

const AWS_SIGNER_MAX_RETRIES: u32 = 5;
const AWS_SINGER_MIN_RETRY_DELAY_MS: u64 = 1000;

/// Error types for `AwsPair`
#[derive(Debug, thiserror::Error)]
pub enum AwsPairError {
    /// AWS Signer Error
    #[error("Error from EthersAwsSigner: {0}")]
    AwsSignerError(#[from] AwsSignerError),
    /// Public key length error
    #[error("EthersAwsSigner returned a bad public key length")]
    PubKeyBadLength,
}

/// A partially implemented `subxt::ext::sp_core::Pair` that
/// will support a remote AWS signer using ECDSA
#[derive(Clone)]
pub struct AwsPair {
    signer: EthersAwsSigner<'static>,
    pubkey: AwsPublic,
    max_retries: u32,
    min_retry_ms: u64,
}

impl AwsPair {
    /// Create a new `AwsPair` from an AWS id
    pub async fn new<T>(id: T) -> Result<Self>
    where
        T: AsRef<str> + Send + Sync,
    {
        let kms_client = get_kms_client().await;
        let signer = EthersAwsSigner::new(kms_client, id, 0)
            .await
            .map_err(AwsPairError::AwsSignerError)?;
        let pubkey = signer
            .get_pubkey()
            .await
            .map_err(AwsPairError::AwsSignerError)?;
        let pubkey = pubkey
            .try_into()
            .map_err(|_| AwsPairError::PubKeyBadLength)?;
        Ok(Self {
            signer,
            pubkey,
            max_retries: AWS_SIGNER_MAX_RETRIES,
            min_retry_ms: AWS_SINGER_MIN_RETRY_DELAY_MS,
        })
    }

    /// Our `AwsPublic` key
    fn public_remote(&self) -> AwsPublic {
        self.pubkey
    }

    /// Try to sign `message` using our remote signer
    async fn try_sign_remote(
        &self,
        message: &[u8],
        delay: Duration,
    ) -> Result<AwsSignature, AwsSignerError> {
        sleep(delay).await;
        self.signer
            .sign_message(message)
            .await
            .map(Into::<AwsSignature>::into)
    }

    /// Try to sign `message` `max_retries` times with an exponential backoff between attempts.
    /// If we hit `max_retries` `panic` since we're unable to return an error here.
    fn sign_remote(&self, message: &[u8]) -> AwsSignature {
        let mut times_attempted = 0;
        let mut delay = Duration::from_millis(0);
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("unable to create tokio::runtime (this should never happen)")
            .block_on(async {
                loop {
                    match self.try_sign_remote(message, delay).await {
                        Ok(signature) => return signature,
                        Err(error) => {
                            times_attempted += 1;
                            delay = Duration::from_millis(self.min_retry_ms.pow(times_attempted));
                            if times_attempted == self.max_retries {
                                panic!(
                                    "giving up after attempting to sign message {} times: {:?}",
                                    times_attempted, error,
                                )
                            }
                        }
                    }
                }
            })
    }
}

/// To make `AwsPair` init from an AWS id while keeping our internal signer
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
impl FromAwsId for AwsPair {
    async fn from_aws_id<T>(id: T) -> Result<Self>
    where
        T: AsRef<str> + Send + Sync,
    {
        AwsPair::new(id).await
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

/// A `subxt::ext::sp_core::Public` key that is compatible with
/// `subxt::ext::sp_core::Pair` and AWS's ECDSA KMS signer
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct AwsPublic(pub [u8; 33]);

impl UncheckedFrom<[u8; 33]> for AwsPublic {
    fn unchecked_from(x: [u8; 33]) -> Self {
        AwsPublic(x)
    }
}

impl Derive for AwsPublic {}

impl CryptoType for AwsPublic {
    type Pair = AwsPair;
}

impl ByteArray for AwsPublic {
    const LEN: usize = 33;
}

impl std::fmt::Display for AwsPublic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.as_ref()))
    }
}

impl AsRef<[u8]> for AwsPublic {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsMut<[u8]> for AwsPublic {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl TryFrom<EthersVerifyingKey> for AwsPublic {
    type Error = ();

    fn try_from(data: EthersVerifyingKey) -> Result<Self, Self::Error> {
        let data = data.to_bytes();
        TryFrom::<&[u8]>::try_from(data.as_slice())
    }
}

impl TryFrom<&[u8]> for AwsPublic {
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

impl TraitPublic for AwsPublic {
    fn to_public_crypto_pair(&self) -> CryptoTypePublicPair {
        CryptoTypePublicPair(ecdsa::CRYPTO_ID, self.as_ref().to_vec())
    }
}

/// A `Signature` that is compatible with `subxt::ext::sp_core::Pair`
/// and AWS's ECDSA KMS signer
#[derive(PartialEq, Eq, Hash)]
pub struct AwsSignature(pub [u8; 65]);

impl From<EthersSignature> for AwsSignature {
    fn from(signature: EthersSignature) -> Self {
        AwsSignature(signature.into())
    }
}

impl From<AwsSignature> for MultiSignature {
    fn from(_x: AwsSignature) -> Self {
        todo!()
    }
}

impl AsRef<[u8]> for AwsSignature {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

/// We only need this to satisfy the associated type on
/// `subxt::ext::sp_core::Pair`, so we'll make it a seed of length zero
type DummySeed = [u8; 0];

/// The trait `subxt::ext::sp_core::Pair` handles signing, verification and the creation
/// of keypairs from local key material (mnemonics, random bytes, etc.). With a remote
/// AWS signer, keypair creation is handled externally so we will only partially implement
/// `Pair` to reflect this.
impl TraitPair for AwsPair {
    type Public = AwsPublic;
    type Seed = DummySeed;
    type Signature = AwsSignature;
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

impl CryptoType for AwsPair {
    type Pair = AwsPair;
}
