//! Factory-bound scalar chromosome bodies. See docs/native-chromosomes.md.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeChromosome {
    pub kind: u8,
    pub body: NativeChromosomeBody,
}

/// Offsets name native storage, not inferred biological meanings. Short fields
/// retain raw bits; byte-to-u32 widening is explicit where observed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeChromosomeBody {
    Empty,
    Byte08 {
        field_08: u8,
    },
    WideByte08 {
        field_08: u32,
    },
    WideShort08 {
        field_08: u32,
    },
    Gills {
        field_08: u32,
        field_0c: u32,
    },
    Horn {
        field_08: u32,
        field_0c: u8,
    },
    Antenna {
        field_08: u16,
        field_0c: u32,
        field_10: u8,
    },
    Reproduction {
        field_08: u32,
        field_0c: u8,
        field_0d: u8,
        field_0e: u8,
        field_10: u32,
    },
    Mammary {
        field_08: u8,
        field_10: u8,
        field_12: u16,
        field_0c: u32,
        field_14: u8,
    },
}

impl NativeChromosome {
    /// Body after the type byte. Unknown factory types return None without
    /// consuming guessed bytes. Known nested bodies remain Unsupported until
    /// their complete gene/color readers are available. Atomic Rust errors do
    /// not emulate Qt failed-stream continuation or native partial writes.
    pub fn read(kind: u8, input: &mut QtReader<'_>) -> io::Result<Option<Self>> {
        match kind {
            4..=14 | 17..=20 => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native chromosome gene/color body not reconstructed",
                ));
            }
            0..=3 | 15 | 16 | 21..=27 => (),
            _ => return Ok(None),
        }
        let mut record = QtReader::new(input.remaining_bytes());
        let body = match kind {
            0 | 1 => NativeChromosomeBody::Empty,
            2 | 3 | 23 => NativeChromosomeBody::WideByte08 {
                field_08: u32::from(record.read_u8()?),
            },
            21 | 24 => NativeChromosomeBody::Byte08 {
                field_08: record.read_u8()?,
            },
            25 => NativeChromosomeBody::WideShort08 {
                field_08: u32::from(record.read_u16()?),
            },
            22 => NativeChromosomeBody::Gills {
                field_08: u32::from(record.read_u8()?),
                field_0c: u32::from(record.read_u8()?),
            },
            16 => NativeChromosomeBody::Horn {
                field_08: u32::from(record.read_u8()?),
                field_0c: record.read_u8()?,
            },
            15 => NativeChromosomeBody::Antenna {
                field_08: record.read_u16()?,
                field_0c: u32::from(record.read_u8()?),
                field_10: record.read_u8()?,
            },
            26 => {
                let field_08 = u32::from(record.read_u8()?);
                let field_0c = record.read_u8()?;
                let packed = record.read_u8()?;
                NativeChromosomeBody::Reproduction {
                    field_08,
                    field_0c,
                    field_0d: packed >> 4,
                    field_0e: packed & 15,
                    field_10: u32::from(record.read_u8()?),
                }
            }
            27 => {
                let packed = record.read_u8()?;
                NativeChromosomeBody::Mammary {
                    field_08: packed & 1,
                    field_10: (packed >> 1) & 1,
                    field_12: record.read_u16()?,
                    field_0c: u32::from(record.read_u8()?),
                    field_14: record.read_u8()?,
                }
            }
            _ => unreachable!("factory classification checked"),
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(Some(Self { kind, body }))
    }
}
