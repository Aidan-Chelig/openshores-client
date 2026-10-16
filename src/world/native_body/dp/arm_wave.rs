//! Native WaveArms/Wave stage, not full action admission.
use super::*;
use crate::world::native_creature::{NativePoseRhythmTimer, native_dh_pose_rhythm};
use std::io;

impl NativeDpInitialization {
    pub fn wave_arms(
        &mut self,
        side: i32,
        pitch_30: f32,
        timer: &mut impl NativePoseRhythmTimer,
    ) -> io::Result<()> {
        if !pitch_30.is_finite() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonfinite Wave owner pitch",
            ));
        }
        self.spread_arms(f32::from_bits(0xbfbde44e), f32::from_bits(0xbe860a92), side)?;
        for part in self
            .parts
            .iter()
            .skip(1)
            .filter(|part| super::spread_arms::ordinary_arm(part))
        {
            if !super::spread_arms::selected(part, side) {
                continue;
            }
            let phase = phase(&part.allocation);
            // Wave intentionally samples AuPose 1, NOT its dispatch code 0x30.
            let wave = native_dh_pose_rhythm(&part.allocation.dna, 1, phase, 1., timer)?;
            let NativeDpPartJoints::Arm {
                splay_4c, lift_48, ..
            } = part.joints
            else {
                return Err(io::Error::other("Wave requires ordinary arm joints"));
            };
            let angles = angles(
                wave,
                pitch_30,
                native_dh_has_wrist_bend(&part.allocation.dna),
            );
            part.rotate_arm_shoulder(&mut self.bones, angles[0])?;
            part.rotate_arm_wrist(&mut self.bones, angles[1])?;
            part.rotate_arm_elbow(&mut self.bones, angles[2])?;
            part.splay_arm_shoulder(&mut self.bones, splay_4c)?;
            part.lift_arm_shoulder(&mut self.bones, lift_48 + angles[3])?;
        }
        Ok(())
    }
}
fn phase(part: &NativeDpAllocatedPart) -> f32 {
    f32::from_bits(match (part.dna.0[1] & 3, part.index) {
        (2, 1) => 0x40490fdb,
        (3, 1) => 0x40060a92,
        (3, 2) => 0x40860a92,
        _ => 0,
    })
}
fn angles(wave: f32, pitch: f32, wrist_bend: bool) -> [f32; 4] {
    let degree = f32::from_bits(0x3c8efa35);
    let swing = 10_f32 * wave;
    [
        (85_f32 + wave) * degree + pitch,
        if wrist_bend {
            wave * f32::from_bits(0x3e32b8c2) - f32::from_bits(0x3dd67750)
        } else {
            f32::from_bits(0xbd0efa35)
        },
        (15_f32 + swing) * degree,
        swing * degree,
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wave_phase_ignores_side_and_angles_keep_native_float_order() {
        for kind in [
            NativeDpPartKind::LeftArmAndHand,
            NativeDpPartKind::RightArmAndHand,
        ] {
            let mut part = NativeDpAllocatedPart {
                kind,
                index: 1,
                dna: NativeDhDna([0; 24]),
            };
            part.dna.0[1] = 2;
            assert_eq!(phase(&part).to_bits(), 0x40490fdb);
            part.dna.0[1] = 3;
            assert_eq!(phase(&part).to_bits(), 0x40060a92);
            part.index = 2;
            assert_eq!(phase(&part).to_bits(), 0x40860a92);
        }
        for wave in [-1., 0., 1.] {
            let a = angles(wave, 0.25, true);
            let b = angles(wave, 0.25, false);
            assert_eq!(a[0], (85_f32 + wave) * f32::from_bits(0x3c8efa35) + 0.25);
            assert_eq!(b[1].to_bits(), 0xbd0efa35);
            assert_eq!(a[2], b[2]);
            assert_eq!(a[3], b[3]);
        }
    }
}
