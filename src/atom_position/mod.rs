//! Packed fields read by native AuAtomPos::Rx, not the reference server's six floats.
//! See docs/native-entity-transforms.md. Frame dispatch/profile selection and
//! application to an existing native matrix are deliberately separate.
use crate::protocol::QtReader;
use std::io;

mod environment;
mod matrix;
mod state;
pub use environment::{ParentEnvironment, compose_environment};
pub use state::{EtherPositionEffects, FreshPosition, NativePositionState, PositionChange};
#[cfg(test)]
mod tests;

/// QDataStream configuration must be established by the enclosing protocol.
/// No default: native float/double overloads alone do not establish wire widths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatEncoding {
    /// Qt stream versions before Qt 4.6: float=4 bytes, double=8 bytes.
    TypeWidths,
    /// Qt 4.6+ SinglePrecision: both overloads consume 4 bytes.
    SinglePrecision,
    /// Qt 4.6+ DoublePrecision: both overloads consume 8 bytes.
    DoublePrecision,
}

impl FloatEncoding {
    fn read(self, input: &mut QtReader<'_>, double_overload: bool) -> io::Result<f64> {
        if self == Self::DoublePrecision || (self == Self::TypeWidths && double_overload) {
            input.read_f64()
        } else {
            input.read_f32().map(f64::from)
        }
    }
}

/// Serialized fields only. Missing coordinates/angles are zero, not previous
/// values. Missing parent is retained as None; enclosing native state decides
/// whether to retain a parent or initialize a fresh AuAtomPos with null parent.
#[derive(Clone, Debug, PartialEq)]
pub struct PackedPosition {
    pub flags: u8,
    pub parent: Option<u32>,
    pub position: [f64; 3],
    pub rotation: [f32; 3],
}

impl PackedPosition {
    /// Consume one big-endian packed record, leaving following fields untouched.
    /// A failed decode does not consume input. Non-finite fields are rejected
    /// as a Rust safety boundary, not a claim that the native reader rejects them.
    pub fn read(input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let flags = record.read_u8()?;
        let parent = if flags & 1 != 0 {
            Some(record.read_u32()?)
        } else {
            None
        };
        let mut rotation = [0.0; 3];
        for (index, angle) in rotation.iter_mut().enumerate() {
            if flags & (0x10 << index) != 0 {
                // Native float overload narrows even in DoublePrecision mode.
                *angle = encoding.read(&mut record, false)? as f32;
            }
        }
        let mut position = [0.0; 3];
        for (index, coordinate) in position.iter_mut().enumerate() {
            if flags & (2 << index) != 0 {
                let value = encoding.read(&mut record, flags & 0x80 != 0)?;
                *coordinate = if flags & 0x80 != 0 {
                    value
                } else {
                    f64::from(value as f32)
                };
            }
        }
        if position.iter().any(|v| !v.is_finite()) || rotation.iter().any(|v| !v.is_finite()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-finite packed atom position",
            ));
        }
        input.skip(input.remaining() - record.remaining())?;
        Ok(Self {
            flags,
            parent,
            position,
            rotation,
        })
    }
}
