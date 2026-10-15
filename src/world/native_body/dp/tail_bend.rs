//! DpTail::Bend: descending joints, inclusive suffixes, then ground correction.
use super::{NativeDpBoneTransforms, NativeDpDimensions, NativeDpPartKind, NativeDpPartState};
use std::io;
mod transform;
use transform::Bend;

impl NativeDpPartState {
    /// Uses the real owner body's segment length. A negative or NaN clearance
    /// disables the native ground-correction pass; zero still enables it.
    pub fn bend_tail(
        &self,
        owner: &mut NativeDpBoneTransforms,
        dimensions: &NativeDpDimensions,
        pitch: f32,
        yaw: f32,
        clearance: f32,
    ) -> io::Result<()> {
        if self.allocation.kind != NativeDpPartKind::Tail {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native tail Bend requires Tail",
            ));
        }
        let length = dimensions.fields_4080_through_40b8()[10];
        let step = (length + length) / 3_f32;
        let segments = self.allocation.dna.0[14] >> 6;
        let end = if segments == 0 {
            17
        } else {
            16 + usize::from(segments) * 2
        };
        let mut origin = self.connect_30;
        if segments != 0 {
            origin[1] -= f64::from(f32::from(segments * 2 - 1) * step);
        }
        let bend = Bend::new(pitch, yaw);
        apply_pass(owner, end, origin, f64::from(step), &bend);
        if clearance >= 0. {
            // Native repeats until every checked joint clears the plane. This
            // Rust budget prevents malformed/degenerate transforms hanging the
            // client; it is not a native iteration limit or a success fallback.
            let correction = Bend::ground();
            for _ in 0..20_000 {
                let mut point = origin;
                let mut penetrates = false;
                for joint in (18..=end).rev() {
                    if transform::project_z(&owner.slots[joint], point, end - joint)
                        < f64::from(-clearance)
                    {
                        penetrates = true;
                        break;
                    }
                    let next_y = f64::from(step) + point[1];
                    point[1] = next_y;
                }
                if !penetrates {
                    return Ok(());
                }
                apply_pass(owner, end, origin, f64::from(step), &correction);
            }
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native tail ground correction exceeded Rust iteration budget",
            ));
        }
        Ok(())
    }
}

fn apply_pass(
    owner: &mut NativeDpBoneTransforms,
    end: usize,
    mut point: [f64; 3],
    step: f64,
    bend: &Bend,
) {
    for joint in (17..=end).rev() {
        let pivot = transform::project(&owner.slots[joint], point);
        let translation = bend.about(pivot);
        for bone in &mut owner.slots[joint..=end] {
            bend.apply(bone, translation);
        }
        point[1] += step;
    }
}

#[cfg(test)]
mod tests;
