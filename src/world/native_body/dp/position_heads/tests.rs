use super::*;

#[derive(Default)]
struct Timer {
    events: Vec<&'static str>,
    fail: bool,
}
impl NativePoseRhythmTimer for Timer {
    fn start(&mut self) -> io::Result<()> {
        self.events.push("start");
        if self.fail {
            Err(io::ErrorKind::Unsupported.into())
        } else {
            Ok(())
        }
    }
    fn msecs_since_reference(&mut self) -> io::Result<i64> {
        self.events.push("sample");
        Ok(12345)
    }
}

#[test]
fn incomplete_initialization_rejects_before_sampling_or_mutating() {
    let mut body = NativeDpInitialization::begin(&NativeDhDna([0; 24]));
    let before = body.bones().clone();
    let mut timer = Timer::default();
    let mut pose = NativeDpBodyPoseState::default();
    pose.fields_3c[1] = 0x31;
    assert_eq!(
        body.position_heads(&pose, 0., 0., &mut timer)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(*body.bones(), before);
    assert!(timer.events.is_empty());
}

#[test]
fn full_width_pose_defaults_and_static_pitch_branches_do_not_sample_timer() {
    let dna = NativeDhDna([0; 24]);
    let base = 0.25;
    let mut timer = Timer::default();
    for second in [false, true] {
        for pose in [0, 3, 5, 0x100, 0x10004, u32::MAX] {
            assert_eq!(pitch(&dna, pose, base, second, &mut timer).unwrap(), base);
        }
        for pose in [4, 0x88] {
            assert_eq!(
                pitch(&dna, pose, base, second, &mut timer).unwrap(),
                base - HALF_PI
            );
        }
        for pose in (0x12..=0x15).chain(0x4c..=0x53) {
            assert_eq!(
                pitch(&dna, pose, base, second, &mut timer).unwrap(),
                -HALF_PI
            );
        }
    }
    assert!(timer.events.is_empty());
}

#[test]
fn animated_branches_use_distinct_head_phases_and_float_stages() {
    let dna = NativeDhDna([0; 24]);
    for pose in [0x17, 0x38, 0x31, 0x32, 0x33, 0x34, 0x6c] {
        for second in [false, true] {
            let phase = match (pose, second) {
                (_, false) => 0.,
                (0x17 | 0x38, true) => f32::from_bits(0x40490fdb),
                _ => f32::from_bits(0x3e860a92),
            };
            let wave = native_dh_pose_rhythm(&dna, pose, phase, 1., &mut Timer::default()).unwrap();
            let expected = if pose == 0x17 || pose == 0x38 {
                let offset = (wave * 10. - 10.) * DEGREE;
                if second {
                    0.25 + (offset - HALF_PI)
                } else {
                    offset + 0.25
                }
            } else if second {
                wave * SMALL_ARC
            } else {
                wave * SMALL_ARC - HALF_PI
            };
            let mut timer = Timer::default();
            assert_eq!(
                pitch(&dna, pose, 0.25, second, &mut timer)
                    .unwrap()
                    .to_bits(),
                expected.to_bits()
            );
            assert_eq!(timer.events, ["start", "sample"]);
        }
    }
}
