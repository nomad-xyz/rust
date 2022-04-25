use crate::{merkle_root_from_branch, MerkleProof};
use ethers::prelude::H256;

/// A merkle proof object. The leaf, its path to the root, and its index in the
/// tree.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Proof<const N: usize> {
    /// The leaf
    pub leaf: H256,
    /// The index
    pub index: usize,
    /// The merkle branch
    #[serde(with = "const_array_serde")]
    pub path: [H256; N],
}

mod const_array_serde {
    use super::H256;
    use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serializer};

    pub fn serialize<S, const N: usize>(item: &[H256; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(N))?;
        for i in item {
            seq.serialize_element(i)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D, const N: usize>(d: D) -> Result<[H256; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<H256> = Deserialize::deserialize(d)?;
        if v.len() != N {
            Err(serde::de::Error::custom(format!(
                "Expected a sequence with {} elements. Got {} elements",
                N,
                v.len()
            )))
        } else {
            let mut h: [H256; N] = [Default::default(); N];
            h.copy_from_slice(&v[..N]);
            Ok(h)
        }
    }
}

impl<const N: usize> MerkleProof for Proof<N> {
    /// Calculate the merkle root produced by evaluating the proof
    fn root(&self) -> H256 {
        merkle_root_from_branch(self.leaf, self.path.as_ref(), N, self.index)
    }
}
