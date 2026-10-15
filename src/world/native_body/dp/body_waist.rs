//! DpBody::RotateWaist's ordered torso/head/arm updates.
use super::*;
use std::io;
impl NativeDpInitialization {
    pub fn rotate_waist(&mut self, angle: f32) -> io::Result<bool> {
        if angle == 0. || native_dh_waist_vertex(&self.dna) == 0 {
            return Ok(false);
        }
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native body RotateWaist requires completed part calculations",
            ));
        }
        let rotation = super::waist::pivot_rotation(self.current_waist_loc()?, angle, true);
        for part in &self.parts {
            match part.allocation.kind {
                NativeDpPartKind::Torso => part.rotate_torso_waist(&mut self.bones, angle)?,
                NativeDpPartKind::HeadAndNeck => {
                    let before = part.current_connect_loc(&self.bones)?;
                    let after = project_attachment(&rotation, before);
                    part.move_head_by(
                        &mut self.bones,
                        std::array::from_fn(|i| after[i] - before[i]),
                    )?;
                    part.rotate_neck(&mut self.bones, angle)?;
                }
                NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand => {
                    let before = part.current_connect_loc(&self.bones)?;
                    let after = project_attachment(&rotation, before);
                    part.move_arm_by(
                        &mut self.bones,
                        std::array::from_fn(|i| after[i] - before[i]),
                    )?;
                }
                NativeDpPartKind::Tail
                | NativeDpPartKind::LeftLegAndFoot
                | NativeDpPartKind::RightLegAndFoot => {}
            }
        }
        Ok(true)
    }
}
fn project_attachment(matrix: &NativeDpBoneTransform, [x, y, z]: [f64; 3]) -> [f64; 3] {
    let m = &matrix.matrix_00;
    let a: [f64; 4] = std::array::from_fn(|i| if x == 0. { 0. } else { m[i] * x });
    let b: [f64; 4] = std::array::from_fn(|i| if y == 0. { 0. } else { y * m[4 + i] });
    let c: [f64; 4] = std::array::from_fn(|i| if z == 0. { 0. } else { z * m[8 + i] });
    let zero = x == 0. && y == 0. && z == 0.;
    let w = if zero {
        ((m[3] + m[7]) + m[11]) + m[15]
    } else {
        ((b[3] + a[3]) + c[3]) + m[15]
    };
    let mut out = if zero {
        [m[12], m[13], m[14]]
    } else {
        std::array::from_fn(|i| ((m[12 + i] + b[i]) + a[i]) + c[i])
    };
    if w != 0. && w != 1. {
        for v in &mut out {
            *v /= w;
        }
    }
    out
}
