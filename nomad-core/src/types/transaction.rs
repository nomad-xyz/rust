use crate::{
    accumulator::NomadProof, ChainCommunicationError, Decode, DoubleUpdate, Encode, Message,
    NomadError, NomadMessage, SignedUpdate, TxOutcome,
};

/// Behavior of transaction submission
#[derive(Debug, Clone, PartialEq)]
pub enum TxDispatchKind {
    /// Block until transaction has either succeeded for failed
    WaitForResult,
    /// Do not block, ignore outcome
    FireAndForget,
}

/// Contract method called for transaction submission
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum NomadMethod {
    /// Submit a signed update for inclusion
    Update(SignedUpdate),
    /// Dispatch a message
    Dispatch(Message),
    /// Submit an improper update for slashingq
    ImproperUpdate(SignedUpdate),
    /// Submit a double update for slashing
    DoubleUpdate(DoubleUpdate),
    /// Dispatch a transaction to prove inclusion of some leaf in the replica.
    Prove(NomadProof),
    /// Trigger processing of a message
    Process(NomadMessage),
    /// Prove a leaf in the replica and then process its message
    ProveAndProcess(NomadProof, NomadMessage),
}

/// Event representing the final state a transaction
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum NomadTxStatus {
    /// NotSent
    NotSent,
    /// Pending
    Pending,
    /// Successful
    Successful,
    /// Final
    Finalized,
}

// TODO(matthew): Move me
/// Convert between ChainCommunicationError and NomadTxStatus
impl From<ChainCommunicationError> for NomadTxStatus {
    fn from(_error: ChainCommunicationError) -> Self {
        unimplemented!() // TODO(matthew):
    }
}

// TODO(matthew): Move me
/// Convert between TxOutcome and NomadTxStatus
impl From<TxOutcome> for NomadTxStatus {
    fn from(_outcome: TxOutcome) -> Self {
        unimplemented!() // TODO(matthew):
    }
}

/// An abstract transaction
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct PersistedTransaction {
    /// The method this transaction will be processed by
    pub method: NomadMethod,
    /// Nonce for ordering
    pub counter: u64,
    /// Transaction status
    pub status: NomadTxStatus,
}

impl PersistedTransaction {
    /// Create a new PersistedTransaction
    pub fn new(method: NomadMethod) -> Self {
        PersistedTransaction {
            method,
            counter: 0,
            status: NomadTxStatus::NotSent,
        }
    }
}

impl From<NomadMethod> for PersistedTransaction {
    fn from(method: NomadMethod) -> Self {
        PersistedTransaction {
            method,
            counter: 0,
            status: NomadTxStatus::NotSent,
        }
    }
}

impl Encode for PersistedTransaction {
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        // We should never encounter an error here outside of development
        let encoded: Vec<u8> = bincode::serialize(&self).expect("bincode serialization error");
        writer.write_all(&(encoded.len() as u64).to_be_bytes())?;
        writer.write_all(&encoded)?;
        Ok(8 + encoded.len())
    }
}

impl Decode for PersistedTransaction {
    fn read_from<R>(reader: &mut R) -> Result<Self, NomadError>
    where
        R: std::io::Read,
    {
        let mut encoded_len = [0u8; 8];
        reader.read_exact(&mut encoded_len)?;
        let encoded_len = u64::from_be_bytes(encoded_len) as usize;

        let mut encoded: Vec<u8> = vec![0; encoded_len];
        reader.read_exact(&mut encoded[..])?;
        // We should never encounter an error here outside of development
        let decoded: PersistedTransaction =
            bincode::deserialize(&encoded).expect("bincode deserialization error");

        Ok(decoded)
    }
}

impl std::fmt::Display for PersistedTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PersistedTransaction {:?} {:?}",
            self.method, self.status,
        )
    }
}
