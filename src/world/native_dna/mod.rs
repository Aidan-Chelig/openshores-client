//! Native DNA wire storage, not decoded creature morphology.
use crate::protocol::QtReader;
use std::io;

mod chromosome;
pub use chromosome::{NativeChromosome, NativeChromosomeBody};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeDhDna(pub [u8; 24]);

impl NativeDhDna {
    /// DhDNA/DhDNA_ receive body. Reject invalid lengths atomically as Rust
    /// safety; native warns then copies regardless of the declared length.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let raw = record
            .read_bytes()?
            .ok_or_else(|| invalid("null DhDNA bytes"))?;
        let mut bytes: [u8; 24] = raw
            .try_into()
            .map_err(|_| invalid("DhDNA requires 24 bytes"))?;
        bytes[5] &= 0x7f;
        bytes[23] = 0;
        input.skip(input.remaining() - record.remaining())?;
        Ok(Self(bytes))
    }
}

/// AuSpecies stores a QByteArray. Null and empty are distinct; interpretation
/// by later native species consumers is not established by its receive method.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeSpecies(pub Option<Vec<u8>>);

impl NativeSpecies {
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let raw = record.read_bytes()?;
        // Rust storage budget, not a reconstructed native limit.
        if raw.is_some_and(|bytes| bytes.len() > 1_048_576) {
            return Err(invalid("native species wire storage budget exceeded"));
        }
        let value = Self(raw.map(<[u8]>::to_vec));
        input.skip(input.remaining() - record.remaining())?;
        Ok(value)
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests;
