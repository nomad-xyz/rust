use async_trait::async_trait;
use color_eyre::Result;
use ethers::core::types::H256;

use crate::{
    accumulator::NomadProof,
    traits::{Common, TxOutcome},
    NomadMessage,
};

/// The status of a message in the replica
pub enum MessageStatus {
    /// Message is unknown
    None,
    /// Message has been proven but not processed
    Proven(H256),
    /// Message has been processed
    Processed,
}
impl From<H256> for MessageStatus {
    fn from(status: H256) -> Self {
        if status.is_zero() {
            return MessageStatus::None;
        }
        if status == H256::from_low_u64_be(2) {
            return MessageStatus::Processed;
        }
        MessageStatus::Proven(status)
    }
}

impl From<[u8; 32]> for MessageStatus {
    fn from(status: [u8; 32]) -> Self {
        let status: H256 = status.into();
        status.into()
    }
}

/// Interface for on-chain replicas
#[async_trait]
pub trait Replica: Common + Send + Sync + std::fmt::Debug {
    /// Return the replica domain ID
    fn local_domain(&self) -> u32;

    /// Return the domain of the replica's linked home
    async fn remote_domain(&self) -> Result<u32, <Self as Common>::Error>;

    /// Dispatch a transaction to prove inclusion of some leaf in the replica.
    async fn prove(&self, proof: &NomadProof) -> Result<TxOutcome, <Self as Common>::Error>;

    /// Trigger processing of a message
    async fn process(&self, message: &NomadMessage) -> Result<TxOutcome, <Self as Common>::Error>;

    /// Prove a leaf in the replica and then process its message
    async fn prove_and_process(
        &self,
        message: &NomadMessage,
        proof: &NomadProof,
    ) -> Result<TxOutcome, <Self as Common>::Error> {
        self.prove(proof).await?;

        Ok(self.process(message).await?)
    }

    /// Fetch the status of a message
    async fn message_status(&self, leaf: H256) -> Result<MessageStatus, <Self as Common>::Error>;

    /// Fetch the confirmation time for a specific root
    async fn acceptable_root(&self, root: H256) -> Result<bool, <Self as Common>::Error>;
}
