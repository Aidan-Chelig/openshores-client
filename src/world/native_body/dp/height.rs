//! DpBody::CGHeight(bool), from actual body field48, not a guessed pose enum.
use super::{NativeDhDna, NativeDpDimensions, NativeDpFootCache};
use crate::world::native_creature::NativeDpBodyPoseState;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Constant(f32),
    HalfWidth,
    Standing(f32),
    RelativeStanding,
    Neck,
    Default,
}

fn kind(pose: u32) -> Kind {
    match pose {
        3 | 5..=12 | 14 | 15 | 57..=73 => Kind::Constant(f32::from_bits(0x3fd5_5555)),
        13 | 22 | 36..=38 | 45 | 54 | 74 | 75 => Kind::Standing(1.),
        16 | 17 => Kind::RelativeStanding,
        19 | 20 | 50 | 51 => Kind::HalfWidth,
        28 => Kind::Standing(0.5),
        40 => Kind::Neck,
        55 => Kind::Constant(0.),
        76..=79 => Kind::Constant(1.5),
        80..=83 => Kind::Constant(5.),
        84..=137 => {
            let bit = 1_u64 << (pose - 84);
            if bit & 0x4e66666333333 != 0 {
                Kind::Constant(f32::from_bits(0x3fd5_5555))
            } else if bit & 0xb199998cccccc != 0 {
                Kind::Standing(1.)
            } else if pose == 137 {
                Kind::Neck
            } else {
                Kind::Default
            }
        }
        _ => Kind::Default,
    }
}

impl NativeDpFootCache<'_> {
    /// Consumes the real body pose's fourth word (native offset48). Constant,
    /// dimension-only and no-leg branches never request mounted foot geometry.
    pub fn pose_height(
        &mut self,
        dna: &NativeDhDna,
        dimensions: &NativeDpDimensions,
        field_4068: i32,
        pose: &NativeDpBodyPoseState,
        include_foot_height: bool,
    ) -> io::Result<f32> {
        calculate(
            dna,
            dimensions,
            pose.fields_3c[3],
            include_foot_height,
            |include| self.standing_height(dna, dimensions, field_4068, include),
        )
    }
}

// SSE MAXSS chooses the second operand on equal or unordered inputs.
fn maxss(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

fn calculate(
    dna: &NativeDhDna,
    dimensions: &NativeDpDimensions,
    pose: u32,
    include: bool,
    mut standing: impl FnMut(bool) -> io::Result<f32>,
) -> io::Result<f32> {
    let d = dimensions.fields_4080_through_40b8();
    let half = d[11] * 0.5;
    match kind(pose) {
        Kind::Constant(value) => Ok(value),
        Kind::HalfWidth => Ok(d[12] * 0.5),
        Kind::Default => Ok(maxss(half, 0.)),
        Kind::Neck => Ok(neck_height(dna, d)),
        operation => {
            let factor = match operation {
                Kind::Standing(factor) => factor,
                Kind::RelativeStanding => {
                    if dna.0[1] & 8 == 0 {
                        1.5
                    } else {
                        f32::from_bits(0x3f2a_aaab)
                    }
                }
                _ => unreachable!(),
            };
            let base = if dna.0[12] & 12 != 0 {
                standing(include)?
            } else if half > 0. {
                half
            } else {
                return Ok(maxss(half, 0.));
            };
            Ok(maxss(half, base * factor))
        }
    }
}

fn neck_height(dna: &NativeDhDna, d: &[f32; 15]) -> f32 {
    let scale = [1., 0.9, 0.8, 0.7, 0.6][usize::from((dna.0[14] & 3) + ((dna.0[5] >> 1) & 1))];
    let candidate = if dna.0[5] & 0x2c != 0 {
        d[11]
    } else if d[12] < d[11] {
        d[12]
    } else {
        if scale * d[11] < d[11] {
            return d[11] * 0.5;
        }
        // Native MINSS also selects the second operand on unordered input.
        if d[12] < d[11] { d[12] } else { d[11] }
    };
    if candidate * scale < d[11] {
        return d[11] * 0.5;
    }
    maxss(d[11] * 0.5, (candidate * scale) * 0.5)
}

#[cfg(test)]
mod tests;
