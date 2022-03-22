use crate::NomadError;
use ethers::prelude::{Signature, SignatureError, H256};
use std::convert::TryFrom;

/// Simple trait for types with a canonical encoding
pub trait Encode {
    /// Write the canonical encoding to the writer
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write;

    /// Serialize to a vec
    fn to_vec(&self) -> Vec<u8> {
        let mut buf = vec![];
        self.write_to(&mut buf).expect("!alloc");
        buf
    }
}

/// Simple trait for types with a canonical encoding
pub trait Decode {
    /// Try to read from some source
    fn read_from<R>(reader: &mut R) -> Result<Self, NomadError>
    where
        R: std::io::Read,
        Self: Sized;
}

impl Encode for Signature {
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.to_vec())?;
        Ok(65)
    }
}

impl Decode for Signature {
    fn read_from<R>(reader: &mut R) -> Result<Self, NomadError>
    where
        R: std::io::Read,
    {
        let mut buf = [0u8; 65];
        let len = reader.read(&mut buf)?;
        if len != 65 {
            Err(SignatureError::InvalidLength(len).into())
        } else {
            Ok(Self::try_from(buf.as_ref())?)
        }
    }
}

impl Encode for H256 {
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        writer.write_all(self.as_ref())?;
        Ok(32)
    }
}

impl Decode for H256 {
    fn read_from<R>(reader: &mut R) -> Result<Self, NomadError>
    where
        R: std::io::Read,
        Self: Sized,
    {
        let mut digest = H256::default();
        reader.read_exact(digest.as_mut())?;
        Ok(digest)
    }
}

impl Encode for u32 {
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.to_be_bytes())?;
        Ok(4)
    }
}

impl Decode for u32 {
    fn read_from<R>(reader: &mut R) -> Result<Self, NomadError>
    where
        R: std::io::Read,
        Self: Sized,
    {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
}

impl Encode for u64 {
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.to_be_bytes())?;
        Ok(8)
    }
}

impl Decode for u64 {
    fn read_from<R>(reader: &mut R) -> Result<Self, NomadError>
    where
        R: std::io::Read,
        Self: Sized,
    {
        let mut buf = [0; 8];
        reader.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }
}

impl<const N: usize> Encode for accumulator::Proof<N> {
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        writer.write_all(self.leaf.as_bytes())?;
        writer.write_all(&self.index.to_be_bytes())?;
        for hash in self.path.iter() {
            writer.write_all(hash.as_bytes())?;
        }
        Ok(32 + 8 + N * 32)
    }
}

impl<const N: usize> Decode for accumulator::Proof<N> {
    fn read_from<R>(reader: &mut R) -> Result<Self, NomadError>
    where
        R: std::io::Read,
        Self: Sized,
    {
        let mut leaf = H256::default();
        let mut index_bytes = [0u8; 8];
        let mut path = [H256::default(); N];

        reader.read_exact(leaf.as_bytes_mut())?;
        reader.read_exact(&mut index_bytes)?;
        for item in &mut path {
            reader.read_exact(item.as_bytes_mut())?;
        }

        let index = u64::from_be_bytes(index_bytes) as usize;

        Ok(Self { leaf, index, path })
    }
}
