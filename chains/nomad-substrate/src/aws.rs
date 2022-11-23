use async_trait::async_trait;
use color_eyre::Result;
use ethers_core::{
    k256::ecdsa::VerifyingKey as EthersVerifyingKey, types::Signature as EthersSignature,
};
use ethers_signers::{AwsSigner as EthersAwsSigner, AwsSignerError, Signer};
use nomad_core::aws::get_kms_client;
use rusoto_kms::KmsClient;
use std::{thread, time::Duration};
use subxt::ext::sp_runtime::MultiSigner;
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
use tokio::{runtime, time::sleep};

const AWS_SIGNER_MAX_RETRIES: u32 = 5;

/// Error types for `AwsPair`
#[derive(Debug, thiserror::Error)]
pub enum AwsPairError {
    /// AWS signer error
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
}

impl AwsPair {
    /// Create a new `AwsPair` from an AWS id
    pub async fn new<T>(id: T) -> Result<Self, AwsPairError>
    where
        T: AsRef<str> + Send + Sync,
    {
        // Shared AWS client
        let kms_client = get_kms_client().await;
        Self::new_with_client(id, kms_client).await
    }

    /// Create a new `AwsPair` from a `rusoto_kms::KmsClient` and an AWS id
    pub async fn new_with_client<T>(
        id: T,
        kms_client: &'static KmsClient,
    ) -> Result<Self, AwsPairError>
    where
        T: AsRef<str> + Send + Sync,
    {
        // Init our remote signer
        let signer = EthersAwsSigner::new(kms_client, id, 0)
            .await
            .map_err(AwsPairError::AwsSignerError)?;
        // Get the pubkey from our remote keypair
        let pubkey = signer
            .get_pubkey()
            .await
            .map_err(AwsPairError::AwsSignerError)?;
        // Map our AWS pubkey to our Substrate-compatible one
        // These are both 33-byte ECDSA Secp256k1 compressed points
        let pubkey = pubkey
            .try_into()
            .map_err(|_| AwsPairError::PubKeyBadLength)?;
        Ok(Self {
            signer,
            pubkey,
            max_retries: AWS_SIGNER_MAX_RETRIES,
        })
    }

    /// Our `AwsPublic` key
    fn public_remote(&self) -> AwsPublic {
        self.pubkey
    }

    /// Try to sign `message` `max_retries` times with an exponential backoff between attempts.
    /// If we hit `max_retries` `panic` since we're unable to return an error here.
    fn sign_remote_sync(&self, message: &[u8]) -> AwsSignature {
        let message = message.to_owned();
        let Self {
            signer,
            max_retries,
            ..
        } = self.clone();
        // We may be running this inside an async func, so we want to grab the current
        // runtime instead of spawning a new one.
        let handle = match runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("unable to create new tokio::runtime (this should never happen)")
                .handle()
                .clone(),
        };
        // We're spawning a new thread here to accommodate `tokio::runtime::Handle::block_on`
        thread::spawn(move || {
            handle.block_on(async {
                let mut error = None;
                for i in 0..max_retries {
                    error = Some(
                        match signer
                            .sign_message(&message)
                            .await
                            .map(Into::<AwsSignature>::into)
                        {
                            Ok(signature) => return signature,
                            Err(error) => error,
                        },
                    );
                    sleep(Duration::from_secs(2u64.pow(i))).await;
                }
                panic!(
                    "giving up after attempting to sign message {} times: {:?}",
                    max_retries, error,
                );
            })
        })
        .join()
        .unwrap() // Let our panic bubble up
    }
}

/// To make `AwsPair` init from an AWS id while keeping our internal signer
/// generic over all `subxt::ext::sp_core::Pair` and `subxt::Config`.
/// This will be implemented as a noop for `subxt::ext::sp_core::ecdsa::Pair`
/// and other core implementations
#[async_trait]
pub trait FromAwsId {
    /// Create an AWS-compatible signer from an AWS id
    async fn from_aws_id<T>(id: T) -> Result<Self, AwsPairError>
    where
        T: AsRef<str> + Send + Sync,
        Self: Sized;
}

#[async_trait]
impl FromAwsId for AwsPair {
    async fn from_aws_id<T>(id: T) -> Result<Self, AwsPairError>
    where
        T: AsRef<str> + Send + Sync,
    {
        AwsPair::new(id).await
    }
}

#[async_trait]
impl FromAwsId for ecdsa::Pair {
    async fn from_aws_id<T: AsRef<str>>(_id: T) -> Result<Self, AwsPairError>
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

impl From<AwsPublic> for MultiSigner {
    fn from(pubkey: AwsPublic) -> Self {
        Self::Ecdsa(ecdsa::Public::from_raw(pubkey.0))
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
        self.sign_remote_sync(message)
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

#[cfg(test)]
mod test {
    use super::*;
    use once_cell::sync::OnceCell;
    use rusoto_mock::{
        MockCredentialsProvider, MockRequestDispatcher, MultipleMockRequestDispatcher,
    };

    static MOCK_KMS_CLIENT: OnceCell<KmsClient> = OnceCell::new();

    fn mock_kms_client() -> &'static KmsClient {
        MOCK_KMS_CLIENT.get_or_init(|| {
            // aws kms get-public-key --key-id <key_id>
            let pubkey_response = r#"{
                "KeyId": "arn:aws:kms:ap-southeast-1:000000000000:key/XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX",
                "PublicKey": "MFYwEAYHKoZIzj0CAQYFK4EEAAoDQgAEhn6q/sPAS/tU0M49HnbT2N/o2ApVcOxg8RZmtbTrQKUZ8t2s6bi2/AJ+OcVbtavZzqCRttJG6kS/pyEa53AytQ==",
                "CustomerMasterKeySpec": "ECC_SECG_P256K1",
                "KeySpec": "ECC_SECG_P256K1",
                "KeyUsage": "SIGN_VERIFY",
                "SigningAlgorithms": [
                    "ECDSA_SHA_256"
                ]
            }"#;
            // aws kms sign --key-id <key_id> --message ZKoCXIP1QLeb/r/mmWQwUS6UHyfxS6KYfu+RJwaRG2c= \
            // --signing-algorithm ECDSA_SHA_256 --message-type DIGEST
            // NB: `message` here is `[0u8; 128]` hashed with `ethers_core::utils::hash_message`
            let sign_response = r#"{
                "KeyId": "arn:aws:kms:ap-southeast-1:000000000000:key/XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX",
                "Signature": "MEYCIQCzqO4YPbzgw9LlRVB+X040Rb+e7rqNMZf2DqWe5SmY+gIhANi0TTSPDM4FUwrY7hRUZsDcBFHptIdUEak/fOod6UQQ",
                "SigningAlgorithm": "ECDSA_SHA_256"
            }"#;
            let request_dispatcher = MultipleMockRequestDispatcher::new([
                MockRequestDispatcher::default().with_body(pubkey_response.clone()),
                MockRequestDispatcher::default().with_body(pubkey_response),
                MockRequestDispatcher::default().with_body(sign_response),
            ]);
            KmsClient::new_with(
                request_dispatcher,
                MockCredentialsProvider,
                Default::default(),
            )
        })
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn it_instantiates_and_signs() {
        let id = "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX";
        let kms_client = mock_kms_client();

        let signer = AwsPair::new_with_client(id, kms_client).await;

        assert!(signer.is_ok());

        let signer = signer.unwrap();
        let message = [0u8; 128];
        let signature = signer.sign(&message);

        assert_eq!(
            base64::encode(signature),
            "s6juGD284MPS5UVQfl9ONEW/nu66jTGX9g6lnuUpmPonS7LLcPMx+qz1JxHrq5k93qqK/PrBTCoWkuGiskz9MSM="
        );
    }
}
