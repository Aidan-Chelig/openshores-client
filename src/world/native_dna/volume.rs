//! Actual chromosome virtual128 arithmetic; not a mesh-volume approximation.
//! Static anchors: docs/evidence/native-chromosome-volume-gnu.txt.
use super::{NativeChromosome, NativeChromosomeBody, NativeChromosomeValue, invalid};
use std::io;

// Original PE constants at 1422b4668, 1422b4660, 1422b4578, 1422b457c.
const SCALE: f64 = f64::from_bits(0x3f6a_e06b_81ae_06b0);
const HALF: f64 = f64::from_bits(0x3fe0_0000_0000_0000);
const C4578: f32 = f32::from_bits(0x4086_0a92);
const C457C: f32 = f32::from_bits(0x4049_0fdb);

fn dimension(raw: u16) -> f32 {
    // Two double multiplies, then one narrowing; not raw * a float scale.
    (f64::from(raw) * SCALE * HALF) as f32
}

impl NativeChromosome {
    /// Native Volume slot for a factory-decoded chromosome. Unreconstructed
    /// overrides fail, rather than silently contribute zero to health/growth.
    pub fn native_volume(&self) -> io::Result<f32> {
        match self.kind {
            8 => Ok(dimension(self.volume_short(0x1e)?)
                * C4578
                * dimension(self.volume_short(0x20)?)
                * dimension(self.volume_short(0x22)?)),
            17 => {
                let radius = dimension(self.volume_short(0x12)?);
                Ok(C457C * radius * radius * (radius / 5.0))
            }
            27 => match self.body {
                NativeChromosomeBody::Mammary {
                    field_12, field_0c, ..
                } => {
                    if field_12 == 0 || field_0c == 1 {
                        return Ok(0.0);
                    }
                    let radius = dimension(field_12);
                    Ok(C4578 * radius * radius * radius * f32::from_bits(0x3f33_3333))
                }
                _ => Err(invalid("Mammary volume requires Mammary storage")),
            },
            9..=11 | 19 | 20 => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "native chromosome {} Volume formula is not reconstructed",
                    self.kind
                ),
            )),
            // All twenty factory vptrs dispatch to AuChromosome::Volume,
            // whose complete body returns positive zero, even Wing/Fin.
            0..=7 | 12..=16 | 18 | 21..=26 => Ok(0.0),
            _ => Err(invalid("chromosome type has no native factory")),
        }
    }

    fn volume_short(&self, offset: u8) -> io::Result<u16> {
        let NativeChromosomeBody::WithGenes { fields, .. } = &self.body else {
            return Err(invalid("chromosome volume requires gene-bearing storage"));
        };
        let mut found = fields.iter().filter(|field| field.offset == offset);
        let value = match found.next().map(|field| &field.value) {
            Some(NativeChromosomeValue::Short(value)) => *value,
            _ => return Err(invalid("missing or wrongly typed chromosome volume short")),
        };
        if found.next().is_some() {
            return Err(invalid("duplicate chromosome volume short"));
        }
        Ok(value)
    }
}

/// AuChromatid::Volume: positive-zero seed and ordered single-precision adds.
/// Callers select the actual chromatid; this does not select/merge DNA strands.
pub fn native_chromatid_volume(chromatid: &[NativeChromosome]) -> io::Result<f32> {
    let mut volume = 0.0_f32;
    for chromosome in chromatid {
        volume += chromosome.native_volume()?;
    }
    Ok(volume)
}

#[cfg(test)]
mod tests;
