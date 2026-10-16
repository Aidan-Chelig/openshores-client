//! Native Walk/Run leg consumers. See docs/native-avatar-locomotion.md.
use super::*;
use crate::world::native_creature::{NativePoseRhythmTimer, native_dh_pose_rhythm};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeDpGait {
    Walk,
    Run,
}

#[derive(Clone, Copy, Debug)]
pub struct NativeDpLegLocomotion {
    pub gait: NativeDpGait,
    pub height: f32,
    pub speed: f32,
    pub selector: i32,
    pub field_4068: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeDpLegLocomotionAngles {
    pub joints: NativeDpLegStandAngles,
    pub splay: f32,
}

impl NativeDpInitialization {
    /// One leg stage only. Caller owns full-body pose dispatch/reset and time.
    pub fn locomote_legs(
        &mut self,
        input: NativeDpLegLocomotion,
        feet: &mut NativeDpFootCache<'_>,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !input.height.is_finite() || !input.speed.is_finite() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonfinite leg locomotion input",
            ));
        }
        let spread = match input.gait {
            NativeDpGait::Walk => 0x3db2b8c2,
            NativeDpGait::Run => 0x3c8efa35,
        };
        self.spread_legs(
            f32::from_bits(spread),
            input.selector,
            input.field_4068,
            feet,
        )?;
        for part in self.parts.iter().skip(1).rev().take_while(|p| {
            matches!(
                p.allocation.kind,
                NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot
            )
        }) {
            let selected = match input.selector {
                0 => part.allocation.index != 0,
                1 => part.allocation.index == 0,
                2 => true,
                _ => false,
            };
            if !selected {
                continue;
            }
            let rhythm = native_dh_pose_rhythm(
                &part.allocation.dna,
                match input.gait {
                    NativeDpGait::Walk => 0x26,
                    NativeDpGait::Run => 0x25,
                },
                phase(&part.allocation, input.gait),
                input.speed,
                timer,
            )?;
            part.limb_bones()?;
            let stand = feet.leg_stand_angles(
                &self.dna,
                &part.allocation.dna,
                &self.dimensions,
                input.field_4068,
                input.height,
            )?;
            let NativeDpPartJoints::Leg { splay_48, .. } = part.joints else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "leg locomotion requires leg joints",
                ));
            };
            let angles = calculate(stand, splay_48, rhythm, input.gait);
            if ![
                angles.joints.hip,
                angles.joints.knee,
                angles.joints.ankle,
                angles.splay,
            ]
            .iter()
            .all(|v| v.is_finite())
            {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "nonfinite native leg locomotion angles",
                ));
            }
            part.rotate_leg_hip(&mut self.bones, angles.joints.hip)?;
            part.rotate_leg_knee(&mut self.bones, angles.joints.knee)?;
            if native_dh_has_ankle_bend(&part.allocation.dna) {
                part.rotate_leg_ankle(&mut self.bones, angles.joints.ankle)?;
            }
            part.splay_leg_hip(&mut self.bones, angles.splay)?;
        }
        Ok(())
    }
}

fn phase(part: &NativeDpAllocatedPart, gait: NativeDpGait) -> f32 {
    let left = part.kind == NativeDpPartKind::LeftLegAndFoot;
    let count = (part.dna.0[12] >> 2) & 3;
    let index = part.index;
    let pi = f32::from_bits(0x40490fdb);
    match (gait, count, index, left) {
        (NativeDpGait::Run, 1, _, true) if native_dh_walks_on_hands(&part.dna) => 0.,
        (NativeDpGait::Run, 2, 1, _) => pi,
        (NativeDpGait::Run, 2, _, _) => 0.,
        (NativeDpGait::Walk, 2, 1, true) => f32::from_bits(0x40c90fdb),
        (NativeDpGait::Walk, 2, 1, false) => pi,
        (_, 3, 1, true) => f32::from_bits(0x40a78d36),
        (_, 3, 1, false) => f32::from_bits(0x40060a92),
        (_, 3, 2, true) => f32::from_bits(0x40ea9280),
        (_, 3, 2, false) => f32::from_bits(0x40860a92),
        (_, _, _, true) => pi,
        _ => 0.,
    }
}

fn calculate(
    stand: NativeDpLegStandAngles,
    splay: f32,
    rhythm: f32,
    gait: NativeDpGait,
) -> NativeDpLegLocomotionAngles {
    // Keep the shipped float/double narrowing order, including its literal
    // double made by extending the float degree-to-radian value.
    let degrees = f64::from(f32::from_bits(0x3c8efa35));
    let swing = match gait {
        NativeDpGait::Walk => 10.,
        NativeDpGait::Run => 40.,
    };
    let hip = (f64::from(stand.hip) + (-3. - f64::from(rhythm) * swing) * degrees) as f32;
    let damp = (f64::from(splay.abs().sin()) * 0.8) as f32;
    let knee_scale = match gait {
        NativeDpGait::Walk => f64::from(stand.knee) * 0.5,
        NativeDpGait::Run => f64::from(stand.knee + f32::from_bits(0x40490fdb)) / 6.,
    };
    let amplitude = (knee_scale * f64::from(1_f32 - damp)) as f32;
    let knee = stand.knee - (amplitude - rhythm * amplitude);
    let reflected = if splay > f32::from_bits(0x3fc90fdb) {
        f32::from_bits(0x40490fdb) - splay
    } else {
        splay
    };
    let sway = match gait {
        NativeDpGait::Walk => f64::from(damp) / 3.,
        NativeDpGait::Run => f64::from(damp) * 0.5,
    };
    let sway = (sway * f64::from(reflected)) as f32;
    NativeDpLegLocomotionAngles {
        joints: NativeDpLegStandAngles {
            hip,
            knee,
            ankle: stand.ankle,
        },
        splay: splay + rhythm * sway,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rhythm_extrema_preserve_ankle_and_change_native_hip_knee_splay() {
        let stand = NativeDpLegStandAngles {
            hip: 0.2,
            knee: 1.,
            ankle: -0.4,
        };
        for gait in [NativeDpGait::Walk, NativeDpGait::Run] {
            let a = calculate(stand, 0.3, -1., gait);
            let b = calculate(stand, 0.3, 1., gait);
            assert_eq!(a.joints.ankle, stand.ankle);
            assert_eq!(b.joints.ankle, stand.ankle);
            assert_eq!(b.joints.knee, stand.knee);
            assert!(a.joints.knee < b.joints.knee);
            assert!(a.joints.hip > b.joints.hip);
            assert!(a.splay < b.splay);
            let swing = if gait == NativeDpGait::Walk { 10. } else { 40. };
            assert_eq!(
                b.joints.hip,
                (f64::from(stand.hip) + (-3. - swing) * f64::from(f32::from_bits(0x3c8efa35)))
                    as f32
            );
        }
    }
    #[test]
    fn native_phase_assignments_include_synchronized_running_pairs_and_hand_walking() {
        let mut part = NativeDpAllocatedPart {
            kind: NativeDpPartKind::LeftLegAndFoot,
            index: 0,
            dna: NativeDhDna([0; 24]),
        };
        part.dna.0[12] = 8;
        assert_eq!(phase(&part, NativeDpGait::Run), 0.);
        part.index = 1;
        assert_eq!(phase(&part, NativeDpGait::Walk).to_bits(), 0x40c90fdb);
        assert_eq!(phase(&part, NativeDpGait::Run).to_bits(), 0x40490fdb);
        part.dna.0[12] = 4;
        part.dna.0[1] = 1;
        assert!(native_dh_walks_on_hands(&part.dna));
        assert_eq!(phase(&part, NativeDpGait::Run), 0.);
        part.dna.0[11] = 0x60;
        assert_eq!(phase(&part, NativeDpGait::Run).to_bits(), 0x40490fdb);
    }
}
