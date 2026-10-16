//! Native DNA wire storage and verified chromosome-volume arithmetic.
//! Full creature morphology/model generation is not implemented here.
use crate::protocol::QtReader;
use std::io;

mod chromosome;
pub use chromosome::{NativeChromosome, NativeChromosomeBody};
mod color;
pub use color::NativeColorQt7;
mod gene;
pub use gene::{NativeGene, NativeGeneChain, NativeGeneSuffix};
mod gene_chromosome;
pub use gene_chromosome::{NativeChromosomeField, NativeChromosomeValue};
mod animal;
pub use animal::{NativeAuDna, NativeDna};
mod volume;
pub use volume::native_chromatid_volume;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeDhDna(pub [u8; 24]);

impl NativeDhDna {
    /// Adapt an already unframed DhDNA byte array to native model storage.
    /// Keep wire/profile selection in the caller; this does not decode an atom.
    pub fn from_received_bytes(raw: &[u8]) -> io::Result<Self> {
        let mut bytes: [u8; 24] = raw
            .try_into()
            .map_err(|_| invalid("DhDNA requires 24 bytes"))?;
        bytes[5] &= 0x7f;
        bytes[23] = 0;
        Ok(Self(bytes))
    }
    /// DhDNA/DhDNA_ receive body. Reject invalid lengths atomically as Rust
    /// safety; native warns then copies regardless of the declared length.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let raw = record
            .read_bytes()?
            .ok_or_else(|| invalid("null DhDNA bytes"))?;
        let dna = Self::from_received_bytes(raw)?;
        input.skip(input.remaining() - record.remaining())?;
        Ok(dna)
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
