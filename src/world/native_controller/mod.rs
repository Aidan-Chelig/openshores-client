//! Native controller wire values; application/interpolation remains separate.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeUnpackedPosition {
    pub parent: u32,
    pub position: [f64; 3],
    pub rotation: [f64; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub enum NativeController {
    AtomPosition(NativeUnpackedPosition),
    Vector([f64; 3]),
    Motion,
    AtomPositionVector {
        position: NativeUnpackedPosition,
        flags: u8,
        field_118: bool,
        vector_120: [f64; 3],
    },
    Pose {
        field_08: f32,
        field_0c: u32,
        field_10: u8,
        field_12: i16,
    },
}

impl NativeController {
    /// Body only: caller already consumed its factory type byte. Fresh factory
    /// state supplies zero for an omitted AtomPosVec vector. Unknown types
    /// return None without consuming a guessed body. Known malformed bodies
    /// leave this reader unchanged as Rust safety, not Qt error emulation.
    pub fn read(kind: u8, input: &mut QtReader<'_>) -> io::Result<Option<Self>> {
        let mut record = QtReader::new(input.remaining_bytes());
        let value = match kind {
            0 => Self::AtomPosition(position(&mut record)?),
            1 => Self::Vector(point(&mut record)?),
            2 => Self::Motion,
            3 => {
                let position = position(&mut record)?;
                let flags = record.read_u8()?;
                let vector_120 = if flags & 2 != 0 {
                    point(&mut record)?
                } else {
                    [0.; 3]
                };
                Self::AtomPositionVector {
                    position,
                    flags,
                    field_118: flags & 1 != 0,
                    vector_120,
                }
            }
            4 => Self::Pose {
                field_08: record.read_f32()?,
                field_0c: u32::from(record.read_u8()?),
                field_10: record.read_u8()?,
                field_12: record.read_i16()?,
            },
            _ => return Ok(None),
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(Some(value))
    }
}
fn position(input: &mut QtReader<'_>) -> io::Result<NativeUnpackedPosition> {
    Ok(NativeUnpackedPosition {
        parent: input.read_u32()?,
        position: point(input)?,
        rotation: point(input)?,
    })
}
fn point(input: &mut QtReader<'_>) -> io::Result<[f64; 3]> {
    Ok([input.read_f64()?, input.read_f64()?, input.read_f64()?])
}

#[cfg(test)]
mod tests;
