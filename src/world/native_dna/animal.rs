use super::{NativeChromosome, NativeDhDna, gene::spend};
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeAuDna {
    pub version: u8,
    pub chromatids: [Vec<NativeChromosome>; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeDna {
    Dh(NativeDhDna),
    Au(NativeAuDna),
}

impl NativeAuDna {
    /// Fresh wire value, atomically decoded as Rust safety, not the native
    /// in-place Clear/append behavior on malformed streams. Version is stored;
    /// none of the verified chromosome/gene readers branches on it.
    pub fn read_qt7(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let version = record.read_u8()?;
        let mut budget = 20_000; // Rust-only aggregate chromosome/gene budget.
        let mut chromatids = [Vec::new(), Vec::new()];
        for chromatid in &mut chromatids {
            for _ in 0..record.read_u16()? {
                spend(&mut budget)?;
                let kind = record.read_u8()?;
                if let Some(chromosome) =
                    NativeChromosome::read_budget(kind, &mut record, &mut budget)?
                {
                    chromatid.push(chromosome);
                }
            }
        }
        input.skip(input.remaining() - record.remaining())?;
        Ok(Self {
            version,
            chromatids,
        })
    }
}

impl NativeDna {
    /// Dynamic DNA includes its discriminator; embedded DhDNA does not.
    /// Unknown types consume only that discriminator and return None.
    pub fn read_qt7(input: &mut QtReader<'_>) -> io::Result<Option<Self>> {
        let mut record = QtReader::new(input.remaining_bytes());
        let value = match record.read_u8()? {
            3 => Some(Self::Dh(NativeDhDna::read(&mut record)?)),
            4 => Some(Self::Au(NativeAuDna::read_qt7(&mut record)?)),
            _ => None,
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests;
