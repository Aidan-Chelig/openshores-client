//! Concrete tail capability readers and Slither(float,float), not pose dispatch.
use super::tail_bend::transform::Bend;
use super::*;
use std::io;

impl NativeDpPartState {
    fn require_tail(&self) -> io::Result<()> {
        if self.allocation.kind == NativeDpPartKind::Tail {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native tail operation requires Tail",
            ))
        }
    }
    pub fn tail_can_bend_laterally(&self) -> io::Result<bool> {
        self.require_tail()?;
        let shape = self.allocation.dna.0[14] & 0x30;
        let form = self.allocation.dna.0[16] & 7;
        Ok(shape != 0
            && match form {
                2 | 4 | 6 => shape != 0x20,
                3 | 7 => shape == 0x30,
                1 => shape == 0x10,
                _ => shape == 0x20,
            })
    }
    pub fn tail_can_bend_vertically(&self) -> io::Result<bool> {
        self.require_tail()?;
        let shape = self.allocation.dna.0[14] & 0x30;
        let form = self.allocation.dna.0[16] & 7;
        Ok(shape != 0
            && match form {
                6 => shape != 0x10,
                7 => shape != 0x30,
                1..=4 => true,
                _ => shape != 0x20,
            })
    }
    /// Descending joints rotate inclusive suffixes; root halves the current
    /// angles and bypasses the middle-joint reversal, exactly as the native loop.
    pub fn slither_tail(
        &self,
        owner: &mut NativeDpBoneTransforms,
        dimensions: &NativeDpDimensions,
        mut pitch: f32,
        mut yaw: f32,
    ) -> io::Result<()> {
        self.require_tail()?;
        let length = dimensions.fields_4080_through_40b8()[10];
        let step = (length + length) / 3.;
        let segments = self.allocation.dna.0[14] >> 6;
        let end = if segments == 0 {
            17
        } else {
            16 + usize::from(segments) * 2
        };
        let middle = if segments == 0 {
            17
        } else {
            (usize::from(segments) * 2 + 33) >> 1
        };
        let mut point = self.connect_30;
        if segments != 0 {
            point[1] -= f64::from(f32::from(segments * 2 - 1) * step);
        }
        for joint in (17..=end).rev() {
            if joint == 17 {
                pitch *= 0.5;
                yaw *= 0.5;
            } else if joint == middle {
                pitch = -pitch;
                yaw = -yaw;
            }
            let pivot = project(&owner.slots[joint], point);
            let bend = Bend::new(pitch, yaw);
            let translation = bend.about_slither(pivot);
            for bone in &mut owner.slots[joint..=end] {
                bend.apply_slither(bone, translation);
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
        std::array::from_fn(|i| ((m[12 + i] + b[i]) + a[i]) + c[i])
    };
    if w != 0. && w != 1. {
        for v in &mut out {
            *v /= w;
        }
    }
    out
}

impl NativeDpInitialization {
    /// Actual DeadTailFacingDown stage: lateral capability gate then Slither.
    pub fn dead_tail_facing_down(&mut self) -> io::Result<()> {
        self.dead_tail(false)
    }
    /// Actual DeadTailFacingLeft stage: vertical capability gate then Slither.
    pub fn dead_tail_facing_left(&mut self) -> io::Result<()> {
        self.dead_tail(true)
    }
    fn dead_tail(&mut self, left: bool) -> io::Result<()> {
        if !self.parts_calculated() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native dead-tail positioning requires completed part calculations",
            ));
        }
        if let Some(tail) = self
            .parts
            .iter()
            .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
        {
            let capable = if left {
                tail.tail_can_bend_vertically()?
            } else {
                tail.tail_can_bend_laterally()?
            };
            if capable {
                let angle = f32::from_bits(0x3e86_0a92);
                tail.slither_tail(
                    &mut self.bones,
                    &self.dimensions,
                    if left { angle } else { 0. },
                    if left { 0. } else { angle },
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
