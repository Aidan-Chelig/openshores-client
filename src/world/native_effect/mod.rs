//! AuEffect receive fields; offset names intentionally avoid invented meanings.
use crate::protocol::QtReader;
use std::io;
mod list;
pub use list::{NativeEffectList, NativeEffectListReceive};

#[derive(Clone, Debug, PartialEq)]
pub struct NativeEffect {
    pub time_70: i64,
    pub vector_28: [f64; 3],
    pub field_64: u32,
    pub field_08: f32,
    pub field_40: f32,
    pub field_68: bool,
    pub field_a8: u32,
    pub vector_48: [f64; 3],
    pub vector_78: [f64; 3],
    pub vector_90: [f64; 3],
    pub field_ac: u8,
    pub field_b0: f32,
    pub field_b4: u8,
    pub field_b8: f32,
}

impl NativeEffect {
    /// Established temporary initialization in RxUpdateBase, not every field
    /// in AuEffect's C++ object or a default for arbitrary observation state.
    pub fn from_base_receive_temporary() -> Self {
        Self {
            time_70: 0,
            vector_28: [0.; 3],
            field_64: 0,
            field_08: 1.,
            field_40: 1.,
            field_68: false,
            field_a8: 0,
            vector_48: [0.; 3],
            vector_78: [0.; 3],
            vector_90: [0.; 3],
            field_ac: 0,
            field_b0: 0.,
            field_b4: 0,
            field_b8: 0.,
        }
    }

    /// Atomically update this reusable value from Qt version-7 type widths.
    /// Explicit clock/adjustment are sampled observations for this effect.
    /// Incomplete Rust input leaves reader/value unchanged, unlike Qt errors.
    pub fn receive(
        &mut self,
        input: &mut QtReader<'_>,
        now_ms: i64,
        adjustment_ms: i64,
    ) -> io::Result<usize> {
        let mut record = QtReader::new(input.remaining_bytes());
        let mut next = self.clone();
        let flags = record.read_u8()?;
        let low = record.read_u16()?;
        next.time_70 = (now_ms.wrapping_add(adjustment_ms) & !0xffff) | i64::from(low);
        if flags & 1 != 0 {
            next.vector_28 = vector(&mut record)?;
        }
        if flags & 2 != 0 {
            next.field_64 = u32::from(record.read_u8()?);
            next.field_08 = record.read_f32()?;
            next.field_40 = record.read_f32()?;
        }
        if flags & 4 != 0 {
            let packed = record.read_u8()?;
            next.field_68 = packed & 0x80 != 0;
            next.field_a8 = u32::from(packed & 0x7f);
        }
        if flags & 8 != 0 {
            next.vector_48 = vector(&mut record)?;
        }
        if flags & 0x10 != 0 {
            next.vector_78 = vector(&mut record)?;
        }
        if flags & 0x20 != 0 {
            next.vector_90 = vector(&mut record)?;
        }
        (next.field_ac, next.field_b0) = if flags & 0x40 != 0 {
            (record.read_u8()?, record.read_f32()?)
        } else {
            (0, 0.)
        };
        (next.field_b4, next.field_b8) = if flags & 0x80 != 0 {
            (record.read_u8()?, record.read_f32()?)
        } else {
            (0, 0.)
        };
        let consumed = input.remaining() - record.remaining();
        input.skip(consumed)?;
        *self = next;
        Ok(consumed)
    }

    /// RxUpdateBase's subset comparison, not structural or bitwise equality.
    /// IEEE comparisons preserve signed-zero equality and NaN non-matches.
    pub fn matches_append_key(&self, other: &Self) -> bool {
        self.time_70 == other.time_70
            && self.field_a8 == other.field_a8
            && self.field_68 == other.field_68
            && self.field_64 == other.field_64
            && self.vector_28 == other.vector_28
            && self.field_08 == other.field_08
            && self.field_40 == other.field_40
    }
}
fn vector(input: &mut QtReader<'_>) -> io::Result<[f64; 3]> {
    Ok([
        f64::from(input.read_f32()?),
        f64::from(input.read_f32()?),
        f64::from(input.read_f32()?),
    ])
}

#[cfg(test)]
mod tests;
