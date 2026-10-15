use super::common::bounded_string;
use crate::{atom_position::FloatEncoding, protocol::QtReader};
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeItemGalaxyLocation {
    pub id_4c: u32,
    /// Actual stored f64 bit patterns, not opaque wire bytes. Keep signed zero
    /// and NaN payloads without changing item observation equality semantics.
    pub point_50_bits: [u64; 3],
    pub string_68: Option<String>,
}
impl NativeItemGalaxyLocation {
    pub fn point_50(&self) -> [f64; 3] {
        self.point_50_bits.map(f64::from_bits)
    }

    pub(super) fn read(input: &mut QtReader<'_>, profile: FloatEncoding) -> io::Result<Self> {
        let id_4c = input.read_u32()?;
        let mut point_50_bits = [0; 3];
        for bits in &mut point_50_bits {
            *bits = match profile {
                FloatEncoding::SinglePrecision => f64::from(input.read_f32()?),
                FloatEncoding::TypeWidths | FloatEncoding::DoublePrecision => input.read_f64()?,
            }
            .to_bits();
        }
        Ok(Self {
            id_4c,
            point_50_bits,
            string_68: bounded_string(input)?,
        })
    }
}

#[cfg(test)]
mod tests;
