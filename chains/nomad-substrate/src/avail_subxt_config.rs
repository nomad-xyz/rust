use anyhow::Result;
use avail::runtime_types::{
    da_control::extensions::check_app_id::CheckAppId,
    frame_system::extensions::{
        check_genesis::CheckGenesis, check_mortality::CheckMortality, check_nonce::CheckNonce,
        check_spec_version::CheckSpecVersion, check_tx_version::CheckTxVersion,
        check_weight::CheckWeight,
    },
    pallet_transaction_payment,
};
use codec::{Codec, Compact, Decode, Encode, EncodeLike, Error as DecodeError, Input};
use parity_util_mem::MallocSizeOf;
use scale_info::TypeInfo;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Debug;
use subxt::{
    sp_core::{self, H256},
    sp_runtime::{
        traits::{BlakeTwo256, Extrinsic, Hash},
        AccountId32, Digest, MultiAddress, MultiSignature,
    },
    Config,
};

use self::avail::runtime_types::{
    merkle::light::LightMerkle, primitive_types::U256, signature::signature::Signature,
};
use crate::avail_subxt_config::avail::runtime_types::nomad_core::update::{SignedUpdate, Update};

#[subxt::subxt(runtime_metadata_path = "metadata/avail.metadata.08.15.22.scale")]
pub mod avail {}

impl From<LightMerkle> for nomad_core::accumulator::NomadLightMerkle {
    fn from(avail_merkle: LightMerkle) -> Self {
        // avail merkle had to be u32 because of scale encoding limitations
        Self::new(avail_merkle.branch, avail_merkle.count as usize)
    }
}

impl From<nomad_core::Update> for Update {
    fn from(nomad_update: nomad_core::Update) -> Self {
        Update {
            home_domain: nomad_update.home_domain,
            previous_root: nomad_update.previous_root,
            new_root: nomad_update.new_root,
        }
    }
}

impl From<ethers_core::types::Signature> for Signature {
    fn from(ethers_signature: ethers_core::types::Signature) -> Self {
        // Signature { r: ethers_signature.r, s: ethers_signature.s, v: ethers_signature.v }
        unimplemented!("TODO: U256 conversions failing")
    }
}

impl From<nomad_core::SignedUpdate> for SignedUpdate {
    fn from(nomad_signed_update: nomad_core::SignedUpdate) -> Self {
        SignedUpdate {
            update: nomad_signed_update.update.into(),
            signature: nomad_signed_update.signature.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AvailConfig;

impl Config for AvailConfig {
    type Index = u32;
    type BlockNumber = u32;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId32;
    type Address = MultiAddress<Self::AccountId, u32>;
    type Header = DaHeader;
    type Signature = MultiSignature;
    type Extrinsic = AvailExtrinsic;
}
// Needed because we want default deserialization for extrinsics coming from Light client
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AvailExtrinsicLight {
    pub app_id: u32,
    pub signature: Option<MultiSignature>,
    pub data: Vec<u8>,
}

impl Into<AvailExtrinsic> for AvailExtrinsicLight {
    fn into(self) -> AvailExtrinsic {
        AvailExtrinsic {
            app_id: self.app_id,
            signature: self.signature,
            data: self.data,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Default, Encode, TypeInfo)]
pub struct AvailExtrinsic {
    pub app_id: u32,
    pub signature: Option<MultiSignature>,
    pub data: Vec<u8>,
}

pub type SignedExtra = (
    CheckSpecVersion,
    CheckTxVersion,
    CheckGenesis,
    CheckMortality,
    CheckNonce,
    CheckWeight,
    pallet_transaction_payment::ChargeTransactionPayment,
    CheckAppId,
);

const EXTRINSIC_VERSION: u8 = 4;
impl Decode for AvailExtrinsic {
    fn decode<I: Input>(input: &mut I) -> Result<AvailExtrinsic, DecodeError> {
        // This is a little more complicated than usual since the binary format must be compatible
        // with substrate's generic `Vec<u8>` type. Basically this just means accepting that there
        // will be a prefix of vector length (we don't need
        // to use this).
        let _length_do_not_remove_me_see_above: Compact<u32> = Decode::decode(input)?;

        let version = input.read_byte()?;

        let is_signed = version & 0b1000_0000 != 0;
        let version = version & 0b0111_1111;
        if version != EXTRINSIC_VERSION {
            return Err("Invalid transaction version".into());
        }
        let (app_id, signature) = if is_signed {
            let _address = <MultiAddress<AccountId32, u32>>::decode(input)?;
            let sig = MultiSignature::decode(input)?;
            let extra = <SignedExtra>::decode(input)?;
            let app_id = extra.7 .0;
            (app_id, Some(sig))
        } else {
            (0, None)
        };

        let section: u8 = Decode::decode(input)?;
        let method: u8 = Decode::decode(input)?;

        let data: Vec<u8> = match (section, method) {
            // TODO: Define these pairs as enums or better yet - make a dependency on substrate enums if possible
            (29, 1) => Decode::decode(input)?,
            (3, 0) => {
                println!("Timestamp: {:?}", <Compact<u64>>::decode(input)?);
                vec![]
            }
            (a, b) => {
                println!("section, method: ({},{})", a, b);
                vec![]
            }
        };

        Ok(Self {
            app_id,
            signature,
            data,
        })
    }
}

impl<'a> Deserialize<'a> for AvailExtrinsic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let r = sp_core::bytes::deserialize(deserializer)?;
        Decode::decode(&mut &r[..])
            .map_err(|e| serde::de::Error::custom(format!("Decode error: {}", e)))
    }
}

impl Extrinsic for AvailExtrinsic {
    type Call = ();
    type SignaturePayload = ();

    fn is_signed(&self) -> Option<bool> {
        Some(self.signature.is_some())
    }

    fn new(_call: Self::Call, _signed_data: Option<Self::SignaturePayload>) -> Option<Self> {
        None
    }
}

impl MallocSizeOf for AvailExtrinsic {
    fn size_of(&self, _ops: &mut parity_util_mem::MallocSizeOfOps) -> usize {
        // self.app_id.size_of(ops)
        // Implement this if necessary
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct KateCommitment {
    /// The merkle root of the extrinsics.
    pub hash: H256,
    /// Plonk commitment.
    pub commitment: Vec<u8>,
    /// Rows
    pub rows: u16,
    /// Cols
    pub cols: u16,
}

impl MallocSizeOf for KateCommitment {
    fn size_of(&self, ops: &mut parity_util_mem::MallocSizeOfOps) -> usize {
        self.hash.size_of(ops)
            + self.commitment.size_of(ops)
            + self.rows.size_of(ops)
            + self.cols.size_of(ops)
    }
}

// impl<'de> Deserialize<'de> for KateCommitment {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let encoded = sp_core::bytes::deserialize(deserializer)?;
//         Decode::decode(&mut &encoded[..]).map_err(|e| serde::de::Error::custom(e.to_string()))
//     }
// }

pub type AppId = u32;

#[derive(
    Debug, PartialEq, Eq, Clone, Encode, Decode, Default, TypeInfo, Serialize, Deserialize,
)]
pub struct DataLookup {
    pub size: u32,
    pub index: Vec<(AppId, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct DaHeader {
    pub parent_hash: H256,
    #[serde(deserialize_with = "number_from_hex")]
    pub number: u32,
    pub state_root: H256,
    pub extrinsics_root: KateCommitment,
    pub digest: Digest,
    pub app_data_lookup: DataLookup,
}

fn number_from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    let without_prefix = buf.trim_start_matches("0x");
    Ok(u32::from_str_radix(without_prefix, 16).unwrap())
}

pub trait Parameter: Codec + EncodeLike + Clone + Eq + Debug {}
impl<T> Parameter for T where T: Codec + EncodeLike + Clone + Eq + Debug {}

impl MallocSizeOf for DaHeader {
    fn size_of(&self, ops: &mut parity_util_mem::MallocSizeOfOps) -> usize {
        self.parent_hash.size_of(ops)
            + self.number.size_of(ops)
            + self.state_root.size_of(ops)
            + self.extrinsics_root.size_of(ops)
            + self.digest.size_of(ops)
    }
}

impl subxt::sp_runtime::traits::Header for DaHeader {
    type Number = u32;

    type Hash = H256;

    type Hashing = BlakeTwo256;

    fn new(
        number: Self::Number,
        extrinsics_root: Self::Hash,
        state_root: Self::Hash,
        parent_hash: Self::Hash,
        digest: Digest,
    ) -> Self {
        Self {
            parent_hash,
            number,
            state_root,
            extrinsics_root: KateCommitment {
                hash: extrinsics_root,
                commitment: vec![],
                rows: 0,
                cols: 0,
            },
            digest,
            app_data_lookup: DataLookup {
                size: 0,
                index: vec![],
            },
        }
    }

    fn number(&self) -> &Self::Number {
        &self.number
    }

    fn set_number(&mut self, number: Self::Number) {
        self.number = number as u32;
    }

    fn extrinsics_root(&self) -> &Self::Hash {
        &self.extrinsics_root.hash
    }

    fn set_extrinsics_root(&mut self, root: Self::Hash) {
        self.extrinsics_root.hash = root;
    }

    fn state_root(&self) -> &Self::Hash {
        &self.state_root
    }

    fn set_state_root(&mut self, root: Self::Hash) {
        self.state_root = root;
    }

    fn parent_hash(&self) -> &Self::Hash {
        &self.parent_hash
    }

    fn set_parent_hash(&mut self, hash: Self::Hash) {
        self.parent_hash = hash;
    }

    fn digest(&self) -> &Digest {
        &self.digest
    }

    fn digest_mut(&mut self) -> &mut Digest {
        &mut self.digest
    }

    fn hash(&self) -> Self::Hash {
        <Self::Hashing as Hash>::hash_of(self)
    }
}

// mod test{
//     use super::AvailExtrinsic;

//     #[test]
//     fn test_decode_xt() {
//         let xt= serde_json::to_string("0xb1040404000492624a4e287a523d93742df2713c7e7e27781fec205405129b9dc60579765f772172ff27e5569aa97dd9219a906e3e263e61027349e98bd9754ecc6918e4c6bd672deb5b4a8d39e1b0d1d428053eaed04adde199bc4391b5f972c8426583b396adc6810189de3e3b5517dea5a8c36755082ec7b7b6649a4e40ca8c42675653cd7acf0bf708916752ae68410af4cb66295322d14589de3e3b5517dea5a8c36755082ec7b7b6649a4e40ca8c42675653cd7acf0bf708916752ae68410af4cb66295322d145010004000806424142453402010000002a30eb040000000005424142450101de248dfabd539a67697acc6a53b4eb8ff62cc8afc5dce5c5bf3eab4fc945293dff23930c1f98adc60693a91dab91a80739cea6dfc80817c3f435f08d69eb87890100000000").unwrap();
//         let x: AvailExtrinsic = serde_json::from_str(&xt).unwrap();
//     }
// }
