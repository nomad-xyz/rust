use ethers::{types::H256, utils::keccak256};

use crate::{utils, Decode, Encode, NomadError};

const NOMAD_MESSAGE_PREFIX_LEN: usize = 76;

/// A full Nomad message between chains
#[derive(Debug, Default, Clone)]
pub struct NomadMessage {
    /// 4   SLIP-44 ID
    pub origin: u32,
    /// 32  Address in home convention
    pub sender: H256,
    /// 4   Count of all previous messages to destination
    pub nonce: u32,
    /// 4   SLIP-44 ID
    pub destination: u32,
    /// 32  Address in destination convention
    pub recipient: H256,
    /// 0+  Message contents
    pub body: Vec<u8>,
}

/// A partial Nomad message between chains
#[derive(Debug, Default, Clone)]
pub struct Message {
    /// 4   SLIP-44 ID
    pub destination: u32,
    /// 32  Address in destination convention
    pub recipient: H256,
    /// 0+  Message contents
    pub body: Vec<u8>,
}

impl Encode for NomadMessage {
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.origin.to_be_bytes())?;
        writer.write_all(self.sender.as_ref())?;
        writer.write_all(&self.nonce.to_be_bytes())?;
        writer.write_all(&self.destination.to_be_bytes())?;
        writer.write_all(self.recipient.as_ref())?;
        writer.write_all(&self.body)?;
        Ok(NOMAD_MESSAGE_PREFIX_LEN + self.body.len())
    }
}

impl Decode for NomadMessage {
    fn read_from<R>(reader: &mut R) -> Result<Self, NomadError>
    where
        R: std::io::Read,
    {
        let mut origin = [0u8; 4];
        reader.read_exact(&mut origin)?;

        let mut sender = H256::zero();
        reader.read_exact(sender.as_mut())?;

        let mut nonce = [0u8; 4];
        reader.read_exact(&mut nonce)?;

        let mut destination = [0u8; 4];
        reader.read_exact(&mut destination)?;

        let mut recipient = H256::zero();
        reader.read_exact(recipient.as_mut())?;

        let mut body = vec![];
        reader.read_to_end(&mut body)?;

        Ok(Self {
            origin: u32::from_be_bytes(origin),
            sender,
            destination: u32::from_be_bytes(destination),
            recipient,
            nonce: u32::from_be_bytes(nonce),
            body,
        })
    }
}

impl NomadMessage {
    /// Convert the message to a leaf
    pub fn to_leaf(&self) -> H256 {
        keccak256(self.to_vec()).into()
    }

    /// Get the encoded destination + nonce
    pub fn destination_and_nonce(&self) -> u64 {
        utils::destination_and_nonce(self.destination, self.nonce)
    }
}

impl std::fmt::Display for NomadMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NomadMessage {}->{}:{}",
            self.origin, self.destination, self.nonce,
        )
    }
}
