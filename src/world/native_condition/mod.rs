//! Verified native condition bodies; see docs/native-conditions.md.
//! List replacement and Unit/Person composition are separate receive stages.
use super::native_dna::{NativeDhDna, NativeDna, NativeSpecies};
use crate::{atom_position::FloatEncoding, protocol::QtReader};
use std::io;

/// Replacement storage, not an additive effect/controller layer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeConditionList(pub Vec<NativeCondition>);

impl NativeConditionList {
    /// Clear before reading count; completed entries survive a later error.
    /// The native signed-short decrement wraps through all 16-bit patterns.
    pub fn receive(&mut self, input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<()> {
        self.0.clear();
        let count = input.read_u16()?;
        for _ in 0..count {
            let kind = input.read_u8()?;
            if let Some(condition) = NativeCondition::read(kind, input, encoding)? {
                self.0.push(condition);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeCondition {
    pub time_08: i64,
    pub body: NativeConditionBody,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NativeConditionBody {
    Pregnant {
        field_10: Option<NativeDna>,
        field_18: u8,
        id_1c: u32,
        field_20: u8,
        field_28: Option<String>,
    },
    Bounty {
        id_10: u32,
        field_18: Option<String>,
        field_20: Option<String>,
        field_28: f64,
    },
    ImplantDna {
        field_10: NativeDhDna,
    },
    ImplantSpecies {
        field_10: NativeSpecies,
    },
    Poison {
        field_10: u8,
        field_11: u8,
    },
    Disease {
        field_10: u8,
        field_11: u8,
        field_12: u8,
    },
    Paralysis {
        field_10: u8,
        field_11: u8,
    },
    Acid {
        field_10: u8,
        field_11: u8,
    },
    Thought {
        field_10: Option<String>,
    },
    Fire,
    Bandage {
        field_10: u8,
    },
    Antitoxin {
        field_10: u8,
    },
}

impl NativeCondition {
    /// Fresh factory body, after the caller consumes its type byte, using the
    /// Qt7 scene layout for nested DNA. None means the native factory returns
    /// null. Errors leave the reader unchanged as Rust
    /// safety, not a claim about Qt malformed-stream or destructor behavior.
    pub fn read(
        kind: u8,
        input: &mut QtReader<'_>,
        encoding: FloatEncoding,
    ) -> io::Result<Option<Self>> {
        match kind {
            1..=5 | 7..=13 => (),
            _ => return Ok(None),
        }
        let mut record = QtReader::new(input.remaining_bytes());
        let time_08 = record.read_i64()?;
        let body = match kind {
            9 => {
                let flags = record.read_u8()?;
                let field_10 = if flags & 0x40 != 0 {
                    NativeDna::read_qt7(&mut record)?
                } else if flags & 0x20 != 0 {
                    None
                } else {
                    Some(NativeDna::Dh(NativeDhDna::read(&mut record)?))
                };
                let field_18 = record.read_u8()?;
                let field_28 = record.read_string()?;
                // Counted list reception constructs fresh conditions. Native
                // omission retains this factory-initialized zero ID; this is
                // not a reusable in-place Pregnant update API.
                let id_1c = if flags & 0x80 != 0 {
                    record.read_u32()?
                } else {
                    0
                };
                NativeConditionBody::Pregnant {
                    field_10,
                    field_18,
                    id_1c,
                    field_20: flags & 3,
                    field_28,
                }
            }
            11 => NativeConditionBody::Bounty {
                id_10: record.read_u32()?,
                field_18: record.read_string()?,
                field_20: record.read_string()?,
                field_28: match encoding {
                    FloatEncoding::SinglePrecision => f64::from(record.read_f32()?),
                    FloatEncoding::TypeWidths | FloatEncoding::DoublePrecision => {
                        record.read_f64()?
                    }
                },
            },
            4 => NativeConditionBody::ImplantDna {
                field_10: NativeDhDna::read(&mut record)?,
            },
            12 => NativeConditionBody::ImplantSpecies {
                field_10: NativeSpecies::read(&mut record)?,
            },
            1 => NativeConditionBody::Poison {
                field_10: record.read_u8()?,
                field_11: nonzero(record.read_u8()?),
            },
            2 => NativeConditionBody::Disease {
                field_10: record.read_u8()?,
                field_11: record.read_u8()?,
                field_12: nonzero(record.read_u8()?),
            },
            3 => NativeConditionBody::Paralysis {
                field_10: record.read_u8()?,
                field_11: nonzero(record.read_u8()?),
            },
            5 => NativeConditionBody::Acid {
                field_10: record.read_u8()?,
                field_11: nonzero(record.read_u8()?),
            },
            7 => NativeConditionBody::Thought {
                field_10: record.read_string()?,
            },
            8 => NativeConditionBody::Fire,
            10 => NativeConditionBody::Bandage {
                field_10: nonzero(record.read_u8()?),
            },
            13 => NativeConditionBody::Antitoxin {
                field_10: nonzero(record.read_u8()?),
            },
            _ => unreachable!("factory classification checked above"),
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(Some(Self { time_08, body }))
    }
}

fn nonzero(value: u8) -> u8 {
    if value == 0 { 1 } else { value }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod pregnant_tests;
