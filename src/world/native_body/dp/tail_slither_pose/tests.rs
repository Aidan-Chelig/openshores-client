use super::*;

struct Timer {
    events: Vec<&'static str>,
    millis: i64,
    fail: bool,
}
impl Default for Timer {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            millis: 12345,
            fail: false,
        }
    }
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
        Ok(self.millis)
    }
}
fn tail(selector: u8, shape: u8, form: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[14] = (selector << 6) | (shape << 4);
    dna.0[16] = form;
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: NativeDpPartKind::Tail,
            index: 0,
            dna,
        },
        connect_30: [0., 2., 0.],
        joints: NativeDpPartJoints::None,
    }
}

#[test]
fn pose_selection_compares_actual_bones_and_timer_order_across_shapes_and_forms() {
    let rhythm_poses = [17, 22, 24, 25, 26, 28, 33, 37, 38, 42, 43, 75];
    for pose in (0..=255).chain([0x10011, u32::MAX]) {
        for shape in 0..4 {
            for form in 0..8 {
                for general in [0, 24, 25, 27, 42, 43, 45, 53, 0x10018, u32::MAX] {
                    let special = [24, 25, 27, 42, 43, 45, 53].contains(&general);
                    let part = tail(2, shape, form);
                    let dims = NativeDpDimensions::from_dna(&part.allocation.dna);
                    let mut actual = NativeDpBoneTransforms::initialized();
                    let mut expected = actual.clone();
                    let mut body = NativeDpBodyPoseState::default();
                    body.fields_3c[3] = general;
                    let mut timer = Timer::default();
                    let wave = if rhythm_poses.contains(&pose) {
                        Some(
                            native_dh_pose_rhythm(
                                &part.allocation.dna,
                                pose,
                                f32::from_bits(0x40490fdb),
                                0.75,
                                &mut timer,
                            )
                            .unwrap()
                                * f32::from_bits(0x3f060a92),
                        )
                    } else if part.tail_can_bend_laterally().unwrap() {
                        Some(f32::from_bits(0x3f1c61aa))
                    } else {
                        None
                    };
                    if let Some(amplitude) = wave
                        && amplitude != 0.
                        && (special || part.tail_can_bend_laterally().unwrap())
                    {
                        let (pitch, yaw) = if special && !part.tail_can_bend_vertically().unwrap() {
                            (amplitude, 0.)
                        } else {
                            (0., amplitude)
                        };
                        part.slither_tail(&mut expected, &dims, pitch, yaw).unwrap();
                    }
                    let events = timer.events.clone();
                    timer.events.clear();
                    part.slither_tail_for_pose(&mut actual, &dims, pose, &body, 0.75, &mut timer)
                        .unwrap();
                    assert_eq!(
                        actual, expected,
                        "pose {pose}, shape {shape}, form {form}, general {general}"
                    );
                    assert_eq!(timer.events, events);
                }
            }
        }
    }
}

#[test]
fn no_segments_zero_wave_and_timer_errors_preserve_bones() {
    let mut part = tail(0, 3, 3);
    let dims = NativeDpDimensions::from_dna(&part.allocation.dna);
    let mut bones = NativeDpBoneTransforms::initialized();
    for bone in &mut bones.slots {
        bone.field_100 = 199;
    }
    let before = bones.clone();
    let body = NativeDpBodyPoseState::default();
    let mut timer = Timer {
        fail: true,
        ..Default::default()
    };
    part.slither_tail_for_pose(&mut bones, &dims, 17, &body, 1., &mut timer)
        .unwrap();
    assert_eq!(bones, before);
    assert!(timer.events.is_empty());
    part.allocation.dna.0[14] |= 0x40;
    assert_eq!(
        part.slither_tail_for_pose(&mut bones, &dims, 17, &body, 1., &mut timer)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(timer.events, ["start"]);
    assert_eq!(bones, before);
    timer.fail = false;
    timer.millis = 2000;
    timer.events.clear();
    part.slither_tail_for_pose(&mut bones, &dims, 17, &body, -2.5, &mut timer)
        .unwrap();
    assert_eq!(timer.events, ["start", "sample"]);
    assert_eq!(bones, before);
}
