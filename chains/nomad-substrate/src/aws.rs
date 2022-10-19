use subxt::error::SecretStringError;
use subxt::ext::{
    sp_core::{
        crypto::{CryptoTypePublicPair, Derive},
        ecdsa, ByteArray, DeriveJunction, Pair as TraitPair, Public as TraitPublic,
    },
    sp_runtime::{CryptoType, MultiSignature, MultiSigner},
};

/// A partially implemented Pair (`subxt::ext::sp_core::Pair`) that
/// will support a remote AWS signer using ECDSA
#[derive(Clone, Debug)]
pub struct Pair;

impl Pair {
    /// Create a new AWS Pair from an AWS id
    pub fn new<T: AsRef<str>>(_id: T) -> Self {
        todo!()
    }

    fn public_remote(&self) -> Public {
        todo!()
    }

    // TODO: Since Pair::sign is infallible, we will have to have a retry count
    // TODO: followed by a panic here if we can't remote sign
    fn sign_remote(&self, message: &[u8]) -> Signature {
        todo!()
    }
}

/// To make `Pair` init from an AWS id while keeping our internal signer
/// generic over all `subxt::ext::sp_core::Pair` and `subxt::Config`.
/// This will be implemented as a noop for `subxt::ext::sp_core::ecdsa::Pair`
/// and other core implementations
pub trait FromAwsId {
    /// Create an AWS-compatible signer from an AWS id
    fn from_aws_id<T: AsRef<str>>(_id: T) -> Self;
}

impl FromAwsId for Pair {
    fn from_aws_id<T: AsRef<str>>(id: T) -> Self {
        Pair::new(id)
    }
}

impl FromAwsId for ecdsa::Pair {
    fn from_aws_id<T: AsRef<str>>(_id: T) -> Self {
        unimplemented!("For compatibility only, ecdsa::Pair cannot be created from an AWS id")
    }
}

/// A `Public` key that is compatible with `subxt::ext::sp_core::Pair`
/// and AWS's ECDSA KMS signer
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Public(pub [u8; 33]);

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

impl TryFrom<&[u8]> for Public {
    type Error = ();

    fn try_from(_value: &[u8]) -> Result<Self, Self::Error> {
        todo!()
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
