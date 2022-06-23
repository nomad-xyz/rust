use crate::{
    accumulator::NomadProof, Decode, Encode, Message, NomadError, NomadMessage,
    SignedFailureNotification, SignedUpdate,
};
use nomad_types::NomadIdentifier;

/// Behavior of transaction submission
#[derive(Debug, Clone, PartialEq)]
pub enum TxDispatchKind {
    /// Block until transaction has either succeeded for failed
    WaitForResult,
    /// Do not block, ignore outcome
    FireAndForget,
}

// TODO(matthew): Add in missing method
/// Contract method called for transaction submission
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum NomadMethod {
    /// Dispatch a message
    Dispatch(Message),
    /// Submit an improper update for slashing
    ImproperUpdate(SignedUpdate),
    /// Dispatch a transaction to prove inclusion of some leaf in the replica.
    Prove(NomadProof),
    /// Trigger processing of a message
    Process(NomadMessage),
    /// Prove a leaf in the replica and then process its message
    ProveAndProcess(NomadProof, NomadMessage),
    /// onlyOwner function. Enrolls replica at given domain chain.
    OwnerEnrollReplica(NomadIdentifier),
    /// onlyOwner function. Unenrolls replica.
    OwnerUnenrollReplica(NomadIdentifier),
    /// onlyOwner function. Sets contract's home to provided home.
    SetHome(NomadIdentifier),
    /// onlyOwner function. Sets permission for watcher at given domain.
    #[allow(missing_docs)]
    SetWatcherPermission {
        watcher: NomadIdentifier,
        domain: u32,
        access: bool,
    },
    /// Unenroll the replica at the given domain provided an updater address
    /// and `SignedFailureNotification` from a watcher
    UnenrollReplica(SignedFailureNotification),
}

// TODO(matthew): Maybe this should be a status enum
/// Event representing the final state a transaction
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum NomadEvent {
    /// Dummy
    Dummy, // TODO(matthew):
    /// Also Dummy
    Dummy2,
}

/// An abstract transaction
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct PersistedTransaction {
    /// The method this transaction will be processed by
    pub method: NomadMethod,
    /// TODO(matthew):
    pub confirm_event: NomadEvent,
}

impl PersistedTransaction {
    /// Create a new PersistedTransaction
    pub fn new(method: NomadMethod) -> Self {
        PersistedTransaction {
            method,
            confirm_event: NomadEvent::Dummy,
        }
    }
}

impl From<NomadMethod> for PersistedTransaction {
    fn from(method: NomadMethod) -> Self {
        PersistedTransaction {
            method,
            confirm_event: NomadEvent::Dummy,
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
            self.method, self.confirm_event,
        )
    }
}
