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
fn tail(selector: u8, shape: u8, form: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[14] = selector << 6 | shape << 4;
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
fn motion_uses_native_pose_speed_arc_and_capability_combinations() {
    for selector in 0..4 {
        for shape in 0..4 {
            for form in 0..8 {
                let part = tail(selector, shape, form);
                for general in [0, 17, 42, 0x10011, u32::MAX] {
                    for speed in [-0.75, 0., 1.5] {
                        for (motion, pose, arc, rate) in [
                            (Motion::Swim, general, f32::from_bits(0x3eb2b8c2), 1.),
                            (Motion::Wave, general, f32::from_bits(0x3e567750), speed),
                            (Motion::Lash, 30, f32::from_bits(0x3f060a92), speed),
                            (Motion::Wag, 47, f32::from_bits(0x3e860a92), 1.),
                        ] {
                            let mut timer = Timer::default();
                            let actual = angles(&part, motion, general, speed, &mut timer).unwrap();
                            if selector == 0 {
                                assert_eq!(actual, None);
                                assert!(timer.events.is_empty());
                                continue;
                            }
                            let mut expected_timer = Timer::default();
                            let wave = native_dh_pose_rhythm(
                                &part.allocation.dna,
                                pose,
                                f32::from_bits(0x40490fdb),
                                rate,
                                &mut expected_timer,
                            )
                            .unwrap();
                            let amplitude = (arc / f32::from(selector * 2)) * wave;
                            let expected = if amplitude == 0. {
                                None
                            } else if !part.tail_can_bend_laterally().unwrap() {
                                Some([amplitude, 0.])
                            } else {
                                Some([
                                    if matches!(motion, Motion::Wag)
                                        && part.tail_can_bend_vertically().unwrap()
                                    {
                                        -f32::from_bits(0x3e860a92)
                                    } else {
                                        0.
                                    },
                                    amplitude,
                                ])
                            };
                            assert_eq!(actual, expected);
                            assert_eq!(timer.events, expected_timer.events);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn timer_failures_and_nan_speed_follow_native_zero_and_sampling_order() {
    let part = tail(1, 3, 3);
    let mut timer = Timer {
        fail: true,
        ..Default::default()
    };
    assert_eq!(
        angles(&part, Motion::Wave, 17, 1., &mut timer)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(timer.events, ["start"]);
    timer.fail = false;
    timer.events.clear();
    let value = angles(&part, Motion::Wave, 17, f32::NAN, &mut timer)
        .unwrap()
        .unwrap();
    assert_eq!(value[0], 0.);
    assert!(value[1].is_nan());
    assert_eq!(timer.events, ["start", "sample"]);
    timer.events.clear();
    assert_eq!(
        angles(&part, Motion::Wave, 0, f32::NAN, &mut timer).unwrap(),
        None
    );
    assert_eq!(timer.events, ["start"]);
}

#[test]
fn incomplete_body_errors_precede_timer_calls_and_all_bone_writes() {
    let mut body = NativeDpInitialization::begin(&NativeDhDna([0; 24]));
    let before = body.bones().clone();
    let pose = NativeDpBodyPoseState::default();
    let mut timer = Timer::default();
    assert_eq!(
        body.swim_tail(&pose, &mut timer).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.wave_tail(&pose, -1., 1., &mut timer)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.lash_tail(-1., 1., &mut timer).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.wag_tail(-1., &mut timer).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.relax_tail(-1.).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert!(timer.events.is_empty());
    assert_eq!(*body.bones(), before);
}
