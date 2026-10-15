//! Verified native condition bodies; see docs/native-conditions.md.
//! List replacement and Unit/Person composition are separate receive stages.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeCondition {
    pub time_08: i64,
    pub body: NativeConditionBody,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeConditionBody {
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
    /// Body only, after the caller consumes its type byte. None means the
    /// native factory returns null. Known but unreconstructed bodies return
    /// Unsupported, never None. Errors leave the reader unchanged as Rust
    /// safety, not a claim about Qt malformed-stream or destructor behavior.
    pub fn read(kind: u8, input: &mut QtReader<'_>) -> io::Result<Option<Self>> {
        match kind {
            4 | 9 | 11 | 12 => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native condition body not reconstructed",
                ));
            }
            1..=3 | 5 | 7 | 8 | 10 | 13 => (),
            _ => return Ok(None),
        }
        let mut record = QtReader::new(input.remaining_bytes());
        let time_08 = record.read_i64()?;
        let body = match kind {
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
