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
fn strike_preserves_float_stages_and_distinct_fallback_signs() {
    for selector in 0..4 {
        for shape in 0..4 {
            for form in 0..8 {
                let mut dna = NativeDhDna([0; 24]);
                dna.0[14] = selector << 6 | shape << 4;
                dna.0[16] = form;
                let tail = NativeDpPartState {
                    allocation: NativeDpAllocatedPart {
                        kind: NativeDpPartKind::Tail,
                        index: 0,
                        dna,
                    },
                    connect_30: [0.; 3],
                    joints: NativeDpPartJoints::None,
                };
                for general in [0, 24, 25, 27, 42, 43, 45, 53, 0x10018, u32::MAX] {
                    let mut timer = Timer::default();
                    let result = strike_angles(&tail, general, &mut timer).unwrap();
                    if selector == 0 {
                        assert_eq!(result, None);
                        assert!(timer.events.is_empty());
                        continue;
                    }
                    let wave = native_dh_pose_rhythm(
                        &tail.allocation.dna,
                        41,
                        f32::from_bits(0x40490fdb),
                        1.,
                        &mut Timer::default(),
                    )
                    .unwrap();
                    let angle =
                        ((wave * 70. + 60.) * f32::from_bits(0x3c8efa35)) / f32::from(selector * 2);
                    let special = [24, 25, 27, 42, 43, 45, 53].contains(&general);
                    let expected = if special && !tail.tail_can_bend_vertically().unwrap() {
                        [angle, 0.]
                    } else if !special && !tail.tail_can_bend_laterally().unwrap() {
                        [-angle, 0.]
                    } else {
                        [0., angle]
                    };
                    assert_eq!(result, Some(expected));
                    assert_eq!(timer.events, ["start", "sample"]);
                }
            }
        }
    }
}

#[test]
fn incomplete_special_motion_fails_before_mutation_or_timer() {
    let mut body = NativeDpInitialization::begin(&NativeDhDna([0; 24]));
    let before = body.bones().clone();
    let pose = NativeDpBodyPoseState::default();
    let mut timer = Timer::default();
    assert_eq!(
        body.strike_tail(&pose, -1., &mut timer).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.writhe_tail_facing_down(&mut timer).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.writhe_tail_facing_left(&mut timer).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.writhe_tail_facing_right(&mut timer)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.writhe_tail_facing_up(&mut timer).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.receptive_tail_facing_up(&mut timer)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.position_tail(&pose, -1., 1., false, &mut timer)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(*body.bones(), before);
    assert!(timer.events.is_empty());
}
