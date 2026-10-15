//! DpBody::OneTimeCalcs, with instruction-level precision and operation order.
//! Static evidence: docs/evidence/native-dp-dimensions-gnu.txt.
use super::{NativeDhDna, native_dh_waist_vertex, native_dp_size_factor};

/// Exact 15 stored f32 fields at4080..40b8. Offsets deliberately avoid inferred
/// anatomy names. This is not complete body/part/animation initialization.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpDimensions {
    fields: [f32; 15],
}

#[cfg(test)]
mod tests;

impl NativeDpDimensions {
    pub fn from_dna(dna: &NativeDhDna) -> Self {
        let raw = &dna.0;
        let size = native_dp_size_factor(dna);
        let factor = |bits: u8, base: f32| f32::from(bits) * 0.0625 + base;
        let extended = |value: f32| (f64::from(value) * 1.3_f64) as f32;
        let mut fields = [0.0; 15];
        fields[0] = extended(factor(raw[2] >> 3, 0.5) * size);
        fields[1] = factor(raw[3] >> 3, 1.0) * 0.25 * (fields[0] * 0.5);
        fields[2] = factor(raw[6] >> 3, 0.5) * size;
        fields[3] = factor(raw[8] >> 3, 1.0) * 0.25 * (fields[2] * 0.5);
        fields[10] = factor(raw[19] >> 3, 0.5) * size;
        fields[9] = native_dh_waist_vertex(dna) as f32 * fields[10];
        let width = f32::from(raw[20] & 31) * 0.0625 * 0.25 * fields[10] + fields[10] / 3.0;
        let a = if raw[5] & 0x10 != 0 && (raw[12] >> 2) & 3 < 2 {
            width * 0.75
        } else {
            width
        };
        let b = if raw[5] & 8 != 0 { width * 0.75 } else { width };
        fields[11] = a;
        fields[12] = b;
        let index = usize::from(((raw[5] >> 1) & 1) + (raw[14] & 3));
        let narrow = [1.0, 0.8, 0.7, 0.6, 0.5][index];
        let second = [1.0, 0.9, 0.8, 0.7, 0.6][index];
        if raw[5] & 0x2c == 0 {
            let smaller = if b < a { b } else { a };
            fields[4] = narrow * smaller;
            fields[5] = smaller * second;
            fields[6] = a.min(b) * second;
        } else {
            fields[4] = b.max(a) * narrow;
            fields[5] = a * second;
            fields[6] = b * second;
        }
        fields[7] = extended(size * factor(raw[15] >> 3, 0.5));
        fields[8] = factor(raw[16] >> 3, 1.0) * 0.25 * (fields[7] * 0.5);
        fields[13] = factor(raw[21] & 31, 0.5) * size;
        fields[14] = factor(raw[22] & 31, 1.0) * 0.25 * (fields[13] * 0.5);
        Self { fields }
    }

    pub fn fields_4080_through_40b8(&self) -> &[f32; 15] {
        &self.fields
    }
}
