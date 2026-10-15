//! Native face overrides; see docs/evidence/native-face-volume-gnu.txt.
use super::*;

impl NativeChromosome {
    pub(super) fn mouth_volume(&self) -> io::Result<f32> {
        let radius = f64::from(self.volume_short(0x20)?) * SCALE * HALF;
        let field14 = f64::from(self.volume_short(0x14)?) * SCALE;
        let field10 = f64::from(self.volume_short(0x10)?) * SCALE;
        let c45a0 = f64::from_bits(0x4010_c152_4000_0000);
        let c45a8 = f64::from_bits(0x3fd0_0000_0000_0000);
        let first_product = c45a0 * radius;
        let first = field10 * first_product * field14 * c45a8;
        let second = (first_product * radius * field14 * c45a8) as f32;
        // Only the second summand is narrowed before the double add.
        Ok((first + f64::from(second)) as f32)
    }

    pub(super) fn nose_volume(&self) -> io::Result<f32> {
        let length = f64::from(self.volume_short(0x12)?) * SCALE;
        let raw_a = self.volume_short(0x14)?;
        let raw_b = self.volume_short(0x16)?;
        let a = f64::from(raw_a) * SCALE * HALF;
        if raw_a == raw_b {
            let c4590 = f64::from_bits(0x4009_21fb_6000_0000);
            return Ok((c4590 * a * a * length) as f32);
        }
        let b = f64::from(raw_b) * SCALE * HALF;
        let (radius, difference) = if a > b { (a, a - b) } else { (b, b - a) };
        let radius = radius as f32;
        let slope = (difference / length) as f32;
        let cone_length = radius / slope;
        let area = C457C * radius * radius;
        let outer = area * cone_length / 3.0_f32;
        let inner = ((f64::from(cone_length) - length) * f64::from(area) / 3.0_f64) as f32;
        // Preserve cancellation and native intermediate arithmetic.
        // Do not replace it with an algebraically simplified frustum volume.
        Ok(outer - inner)
    }
}
