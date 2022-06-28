use async_trait::async_trait;
use color_eyre::Result;
use ethers::core::types::H256;

use crate::{
    accumulator::NomadProof,
    traits::{ChainCommunicationError, Common, TxOutcome},
    CommonTxHandling, CommonTxSubmission, NomadMessage, TxContractStatus, TxDispatchKind,
    TxEventStatus, TxForwarder,
};

/// The status of a message in the replica
#[repr(u8)]
pub enum MessageStatus {
    /// Message is unknown
    None = 0,
    /// Message has been proven but not processed
    Proven = 1,
    /// Message has been processed
    Processed = 2,
}

/// Interface for on-chain replicas
#[async_trait]
pub trait Replica: Common + Send + Sync + std::fmt::Debug {
    /// Return the replica domain ID
    fn local_domain(&self) -> u32;

    /// Return the domain of the replica's linked home
    async fn remote_domain(&self) -> Result<u32, ChainCommunicationError>;

    /// Fetch the status of a message
    async fn message_status(&self, leaf: H256) -> Result<MessageStatus, ChainCommunicationError>;

    /// Fetch the confirmation time for a specific root
    async fn acceptable_root(&self, root: H256) -> Result<bool, ChainCommunicationError>;
}

/// Interface for chain-agnostic tx submission used by the replica
#[async_trait]
pub trait ReplicaTxHandling: CommonTxHandling + Replica + Send + Sync + std::fmt::Debug {
    /// Dispatch a transaction to prove inclusion of some leaf in the replica.
    async fn prove(
        &self,
        proof: &NomadProof,
        dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError>;

    /// Trigger processing of a message
    async fn process(
        &self,
        message: &NomadMessage,
        dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError>;

    /// Prove a leaf in the replica and then process its message
    async fn prove_and_process(
        &self,
        message: &NomadMessage,
        proof: &NomadProof,
        dispatch_kind: TxDispatchKind,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.prove(proof, dispatch_kind.clone()).await?;

        Ok(self.process(message, dispatch_kind).await?)
    }
}

/// Interface for chain-specific tx submission used by the replica
#[async_trait]
pub trait ReplicaTxSubmission:
    CommonTxSubmission
    + TxForwarder
    + TxEventStatus
    + TxContractStatus
    + Replica
    + Send
    + Sync
    + std::fmt::Debug
{
    /// Dispatch a transaction to prove inclusion of some leaf in the replica.
    async fn prove(&self, proof: &NomadProof) -> Result<TxOutcome, ChainCommunicationError>;

    /// Trigger processing of a message
    async fn process(&self, message: &NomadMessage) -> Result<TxOutcome, ChainCommunicationError>;

    /// Prove a leaf in the replica and then process its message
    async fn prove_and_process(
        &self,
        message: &NomadMessage,
        proof: &NomadProof,
    ) -> Result<TxOutcome, ChainCommunicationError> {
        self.prove(proof).await?;

        Ok(self.process(message).await?)
    }
}
