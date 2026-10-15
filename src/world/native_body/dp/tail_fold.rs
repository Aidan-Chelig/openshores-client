//! DpTail::Fold's joint bends and compiled root-quarter-turn coefficients.
use super::tail_bend::transform::Bend;
use super::*;
use std::io;

impl NativeDpPartState {
    /// `field_3a` is the actual owner's byte3a, NOT pose mode6c. Production
    /// ownership of that field remains separate from this concrete operation.
    pub fn fold_tail(
        &self,
        owner: &mut NativeDpBoneTransforms,
        dimensions: &NativeDpDimensions,
        field_3a: bool,
    ) -> io::Result<()> {
        if self.allocation.kind != NativeDpPartKind::Tail {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native Fold requires Tail",
            ));
        }
        let segments = self.allocation.dna.0[14] >> 6;
        if segments == 0 {
            return Ok(());
        }
        let joints = segments * 2;
        let angle = f32::from_bits(0x3f490fdb) / f32::from(joints);
        if angle == 0. {
            return Ok(());
        }
        let lateral = self.tail_can_bend_laterally()?;
        let (pitch, yaw) = if lateral {
            (0., if field_3a { -angle } else { angle })
        } else {
            (-angle, 0.)
        };
        let length = dimensions.fields_4080_through_40b8()[10];
        let step = (length + length) / 3.;
        let mut point = self.connect_30;
        point[1] -= f64::from(f32::from(joints - 1) * step);
        let end = 16 + usize::from(joints);
        for joint in (17..=end).rev() {
            let bend = if joint == 17 {
                // Native constant-folded sin/cos of widened f32 +/-pi/2.
                let s = 0.999999999999999;
                let c = -4.371139000186241e-08;
                if lateral {
                    Bend::from_sin_cos(0., 1., if field_3a { -s } else { s }, c)
                } else {
                    Bend::from_sin_cos(-s, c, 0., 1.)
                }
            } else {
                Bend::new(pitch, yaw)
            };
            let pivot = project(&owner.slots[joint], point);
            let translation = bend.about_fold(pivot);
            for bone in &mut owner.slots[joint..=end] {
                bend.apply(bone, translation);
            }
            point[1] += f64::from(step);
        }
        Ok(())
    }
}

fn project(bone: &NativeDpBoneTransform, point: [f64; 3]) -> [f64; 3] {
    let m = &bone.matrix_00;
    let [x, y, z] = point;
    let a: [f64; 4] = std::array::from_fn(|i| if x == 0. { 0. } else { m[i] * x });
    let b: [f64; 4] = std::array::from_fn(|i| if y == 0. { 0. } else { m[4 + i] * y });
    let c: [f64; 4] = std::array::from_fn(|i| if z == 0. { 0. } else { m[8 + i] * z });
    let zero = point.iter().all(|v| *v == 0.);
    let w = if zero {
        ((m[3] + m[7]) + m[11]) + m[15]
    } else {
        ((b[3] + a[3]) + c[3]) + m[15]
    };
    let mut out = if zero {
        [m[12], m[13], m[14]]
    } else {
        [
            ((m[12] + b[0]) + a[0]) + c[0],
            ((b[1] + a[1]) + c[1]) + m[13],
            ((b[2] + a[2]) + c[2]) + m[14],
        ]
    };
    if w != 0. && w != 1. {
        for v in &mut out {
            *v /= w;
        }
    }
    out
}

impl NativeDpInitialization {
    /// Concrete body FoldTail stage; no complete PositionTail dispatch implied.
    pub fn fold_tail(&mut self, field_3a: bool) -> io::Result<()> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native FoldTail requires completed part calculations",
            ));
        }
        if let Some(tail) = self
            .parts
            .iter()
            .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
        {
            tail.fold_tail(&mut self.bones, &self.dimensions, field_3a)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
