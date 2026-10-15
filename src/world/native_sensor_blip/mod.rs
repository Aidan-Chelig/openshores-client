//! Minimal sensor blips are not full +158 bodies. See native-sensor-blips.md.
use crate::{atom_position::FloatEncoding, protocol::QtReader};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeSensorBlipMin {
    pub kind: u8,
    pub id_08: u32,
    pub position_10: [f64; 3],
    pub body: NativeSensorBlipMinBody,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeSensorBlipMinBody {
    Base,
    Empire {
        field_2c: u8,
    },
    Animal {
        field_2c: u8,
        field_40: u8,
        field_28: u32,
    },
    Bd {
        field_2c: u8,
        field_34: u8,
    },
}

impl NativeSensorBlipMin {
    /// After the type byte, dispatch the actual factory-bound +190 reader.
    /// Unknown factory types return None without guessed body consumption.
    /// Fresh body errors are atomic as Rust safety, not Qt error parity.
    pub fn read(
        kind: u8,
        input: &mut QtReader<'_>,
        encoding: FloatEncoding,
    ) -> io::Result<Option<Self>> {
        if !(1..=43).contains(&kind) {
            return Ok(None);
        }
        let mut record = QtReader::new(input.remaining_bytes());
        let id_08 = record.read_u32()?;
        let mut position_10 = [0.; 3];
        for value in &mut position_10 {
            *value = f64::from(match encoding {
                FloatEncoding::DoublePrecision => record.read_f64()? as f32,
                FloatEncoding::TypeWidths | FloatEncoding::SinglePrecision => record.read_f32()?,
            });
        }
        let body = match kind {
            2 => {
                let field_2c = record.read_u8()?;
                let flags = record.read_u8()?;
                NativeSensorBlipMinBody::Animal {
                    field_2c,
                    field_40: (flags >> 1) & 1,
                    field_28: u32::from((!flags) & 1),
                }
            }
            32..=37 | 39 => NativeSensorBlipMinBody::Bd {
                field_2c: record.read_u8()?,
                field_34: record.read_u8()?,
            },
            7 | 9 | 10 | 13..=15 | 21 | 22 | 28 | 30 | 31 | 40..=43 => {
                NativeSensorBlipMinBody::Base
            }
            _ => NativeSensorBlipMinBody::Empire {
                field_2c: record.read_u8()?,
            },
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(Some(Self {
            kind,
            id_08,
            position_10,
            body,
        }))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeSensorBlipMinList(pub Vec<NativeSensorBlipMin>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "inspect unknown_type before continuing the enclosing packet"]
pub struct NativeSensorBlipMinReceive {
    pub declared_count: u16,
    pub unknown_type: Option<u8>,
    pub unread_entries: u16,
    pub consumed_bytes: usize,
}

impl NativeSensorBlipMinList {
    /// Destructive replacement before count. A null factory stops the list,
    /// reporting its type and unread count; do not continue/skip an opaque body.
    /// Earlier completed appends survive a later Rust error. Qt failed-stream
    /// continuation and destructor details are not emulated.
    pub fn receive(
        &mut self,
        input: &mut QtReader<'_>,
        encoding: FloatEncoding,
    ) -> io::Result<NativeSensorBlipMinReceive> {
        let length = input.remaining();
        self.0.clear();
        let declared_count = input.read_u16()?;
        for index in 0..declared_count {
            let kind = input.read_u8()?;
            let Some(value) = NativeSensorBlipMin::read(kind, input, encoding)? else {
                return Ok(NativeSensorBlipMinReceive {
                    declared_count,
                    unknown_type: Some(kind),
                    unread_entries: declared_count - index - 1,
                    consumed_bytes: length - input.remaining(),
                });
            };
            self.0.push(value);
        }
        Ok(NativeSensorBlipMinReceive {
            declared_count,
            unknown_type: None,
            unread_entries: 0,
            consumed_bytes: length - input.remaining(),
        })
    }
}

#[cfg(test)]
mod tests;
