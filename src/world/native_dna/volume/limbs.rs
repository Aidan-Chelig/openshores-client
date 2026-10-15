//! Native torso/limb overrides; docs/evidence/native-limb-volume-gnu.txt.
use super::*;

fn full(raw: u16) -> f32 {
    (f64::from(raw) * SCALE) as f32
}

#[cfg(test)]
mod tests;

// Both limb methods use this exact f32 sequence. Native uses the larger
// radius's area in BOTH cone terms; this is not a textbook frustum formula.
fn segment(a: u16, b: u16, length: f32) -> f32 {
    let x = full(a);
    let y = full(b);
    if a == b {
        return C457C * x * x * length;
    }
    let (radius, delta) = if x > y { (x, x - y) } else { (y, y - x) };
    let slope = delta / length;
    let height = radius / slope;
    let area = C457C * radius * radius;
    let outer = area * height / 3.0;
    let inner = (height - length) * area / 3.0;
    outer - inner
}

impl NativeChromosome {
    pub(super) fn torso_volume(&self) -> io::Result<f32> {
        let a = dimension(self.volume_short(0x14)?) * C457C * dimension(self.volume_short(0x12)?);
        let b = dimension(self.volume_short(0x1e)?) * C457C * dimension(self.volume_short(0x1c)?);
        let first = full(self.volume_short(0x10)?) * a;
        let second = full(self.volume_short(0x1a)?) * b + first;
        let third = full(self.volume_short(0x16)?) * (b * 0.25) + second;
        Ok(full(self.volume_short(0x18)?) * (a * 0.25) + third)
    }

    pub(super) fn arm_volume(&self) -> io::Result<f32> {
        let field18 = self.volume_short(0x18)?;
        let field10 = self.volume_short(0x10)?;
        let radius = full(field18);
        let mut volume = radius * C4578 * radius * radius + 0.0;
        volume += segment(field18, field10, full(self.volume_short(0x1e)?));
        volume += segment(
            field10,
            self.volume_short(0x22)?,
            full(self.volume_short(0x12)?),
        );
        let end = full(self.volume_short(0x14)?) * 0.5;
        Ok(C4578 * end * end * end * 0.5 + volume)
    }

    pub(super) fn leg_volume(&self) -> io::Result<f32> {
        let field14 = full(self.volume_short(0x14)?);
        let field1c = self.volume_short(0x1c)?;
        let field20 = self.volume_short(0x20)?;
        let radius = full(field1c);
        let expanded14 = f32::from_bits(0x3f99_999a) * field14;
        let mut volume = if field14 > radius || expanded14 > radius {
            field14 * (radius * C4578) * expanded14 * 0.5 + 0.0
        } else {
            0.0
        };
        volume += segment(field1c, field20, full(self.volume_short(0x24)?));
        volume += segment(
            field20,
            self.volume_short(0x12)?,
            full(self.volume_short(0x16)?),
        );
        let tail = full(self.volume_short(0x1a)?)
            * 0.5
            * C4578
            * (full(self.volume_short(0x18)?) * 0.5)
            * full(self.volume_short(0x10)?)
            * 0.5;
        Ok(tail + volume)
    }
}
