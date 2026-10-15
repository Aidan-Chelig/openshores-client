//! Actual part OneTimeCalcs and its partial writes, not complete body animation.
use super::*;
use std::io;
mod arm;
mod leg;

#[derive(Clone, Debug, PartialEq)]
pub enum NativeDpPartJoints {
    None,
    Head {
        field_50: [f64; 3],
    },
    Arm {
        field_58: [f64; 3],
        field_70: [f64; 3],
        field_88: [f64; 3],
    },
    Leg {
        field_50: [f64; 3],
        field_68: [f64; 3],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpPartState {
    pub allocation: NativeDpAllocatedPart,
    pub connect_30: [f64; 3],
    pub joints: NativeDpPartJoints,
}

impl NativeDpPartState {
    fn allocated(allocation: NativeDpAllocatedPart) -> Self {
        use NativeDpPartKind::*;
        let joints = match allocation.kind {
            HeadAndNeck => NativeDpPartJoints::Head { field_50: [0.; 3] },
            LeftArmAndHand | RightArmAndHand => NativeDpPartJoints::Arm {
                field_58: [0.; 3],
                field_70: [0.; 3],
                field_88: [0.; 3],
            },
            LeftLegAndFoot | RightLegAndFoot => NativeDpPartJoints::Leg {
                field_50: [0.; 3],
                field_68: [0.; 3],
            },
            Torso | Tail => NativeDpPartJoints::None,
        };
        Self {
            allocation,
            connect_30: [0.; 3],
            joints,
        }
    }

    pub(super) fn calculate(
        &mut self,
        dimensions: &NativeDpDimensions,
        hand_length: &mut impl FnMut() -> io::Result<f32>,
    ) -> io::Result<()> {
        use NativeDpPartKind::*;
        let raw = &self.allocation.dna.0;
        let length = dimensions.fields_4080_through_40b8()[10];
        let waist = native_dh_waist_vertex(&self.allocation.dna);
        match self.allocation.kind {
            Torso => {} // Actual virtual10 body is a single ret.
            Tail => {
                self.connect_30 = [
                    0.,
                    f64::from((i32::from(raw[20] >> 5) - waist as i32) as f32 * -length),
                    0.,
                ]
            }
            HeadAndNeck => {
                let base = f64::from(waist as f32 * length);
                self.connect_30 = [0., base, 0.];
                let count = (raw[14] & 3) + ((raw[5] >> 1) & 1);
                let y = if count == 0 {
                    base
                } else {
                    let step = length / 2.5_f32;
                    (base + f64::from(f32::from(count) * step)) - f64::from(step) * 0.25
                };
                self.joints = NativeDpPartJoints::Head {
                    field_50: [0., y, 0.],
                };
            }
            LeftArmAndHand | RightArmAndHand => self.calculate_arm(dimensions, hand_length)?,
            LeftLegAndFoot | RightLegAndFoot => self.calculate_leg(dimensions),
        }
        Ok(())
    }
}

/// DpBody::Init's allocation, bone reset, dimensions and part calculations.
/// This deliberately does not claim full constructor/pose/render completion;
/// the remaining body runtime state must be composed too.
#[derive(Clone, Debug)]
pub struct NativeDpInitialization {
    pub(super) dna: NativeDhDna,
    pub(super) dimensions: NativeDpDimensions,
    pub(super) parts: Vec<NativeDpPartState>,
    pub(super) bones: NativeDpBoneTransforms,
    next_part: usize,
}

impl NativeDpInitialization {
    pub fn begin(dna: &NativeDhDna) -> Self {
        Self {
            dna: dna.clone(),
            dimensions: NativeDpDimensions::from_dna(dna),
            parts: native_allocate_dp_parts(dna)
                .into_iter()
                .map(NativeDpPartState::allocated)
                .collect(),
            bones: NativeDpBoneTransforms::initialized(),
            next_part: 0,
        }
    }
    pub fn dimensions(&self) -> &NativeDpDimensions {
        &self.dimensions
    }
    pub fn bones(&self) -> &NativeDpBoneTransforms {
        &self.bones
    }
    pub fn parts(&self) -> &[NativeDpPartState] {
        &self.parts
    }
    pub fn parts_calculated(&self) -> bool {
        self.next_part == self.parts.len()
    }

    /// Earlier parts and the failing arm's pre-HandLength writes survive a
    /// resource error. Resume retries that part; body DNA (not modified per-arm
    /// DNA) is passed to the actual owner-body hand provider, with mode=false.
    pub fn finish_part_calculations(
        &mut self,
        field_4068: i32,
        cache: &mut NativeDpHandCache<'_>,
    ) -> io::Result<()> {
        while self.next_part < self.parts.len() {
            self.parts[self.next_part].calculate(&self.dimensions, &mut || {
                cache.hand_length(&self.dna, field_4068, false)
            })?;
            self.next_part += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
