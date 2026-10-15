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

// Fixture roles are tied to both complete jump tables by the Python verifier.
fn expected(role: &str, base: f32, general: u32) -> (f32, Option<f32>, bool) {
    let nonzero = base != 0.;
    match role {
        "none" => (0., None, false),
        "fixed" => (f32::from_bits(0x3fc25c20), None, false),
        "fallback" => (
            if nonzero {
                base
            } else {
                f32::from_bits(0x3fbde44f)
            },
            None,
            false,
        ),
        "negative" => (if nonzero { base } else { -0.17453292 }, None, false),
        "quarter" => (f32::from_bits(0x3fb2b8c3), None, false),
        "halfpi" => (std::f32::consts::FRAC_PI_2, None, false),
        "small" => (
            if nonzero {
                base
            } else {
                f32::from_bits(0x3fbde44f)
            },
            Some(0.02617994),
            false,
        ),
        "23" => (
            if nonzero { base } else { 0.2617994 },
            Some(0.034906585),
            true,
        ),
        "24" => (
            if nonzero { 0.2617994 } else { 0.13962634 },
            Some(0.04363323),
            true,
        ),
        "25" => (
            if nonzero { 0.08726646 } else { 0.017453292 },
            Some(0.061086524),
            true,
        ),
        "27" => (
            if nonzero { 0.43633232 } else { 0.2617994 },
            Some(0.02617994),
            true,
        ),
        "28" => (
            if nonzero {
                base * 0.5
            } else {
                std::f32::consts::FRAC_PI_4
            },
            Some(0.05235988),
            false,
        ),
        "29" => (f32::from_bits(0x3fcb4bc4), Some(0.02617994), true),
        "37" => (
            if nonzero {
                base
            } else {
                f32::from_bits(0x3fa78d36)
            },
            Some(0.04363323),
            false,
        ),
        "41" => {
            let b = match general {
                24 | 25 | 27 | 53 => base - 0.17453292,
                42 | 43 | 45 => 0.,
                _ => std::f32::consts::FRAC_PI_4,
            };
            (b, Some(0.43633232), true)
        }
        "42" => (0., Some(0.02617994), true),
        "43" => (0., Some(0.04363323), true),
        "45" => (
            if nonzero {
                0.
            } else {
                f32::from_bits(0x3fc25c20)
            },
            Some(0.008726646),
            true,
        ),
        "46" => (f32::from_bits(0x3fc49809), Some(0.017453292), true),
        "tiny" => (0., Some(0.008726646), true),
        "50" => (
            if nonzero { base } else { -0.17453292 },
            Some(0.008726646),
            true,
        ),
        _ => panic!("unknown fixture role {role}"),
    }
}

#[test]
fn every_native_dispatch_entry_matches_source_bound_roles_and_timer_order() {
    let fixture = include_str!("../../../../../tests/fixtures/native_dp_position_torso.tsv");
    let mut dna = NativeDhDna([0; 24]);
    dna.0[17] = 7;
    for line in fixture.lines().filter(|l| !l.starts_with('#')) {
        let mut fields = line.split('\t');
        let selected: u32 = fields.next().unwrap().parse().unwrap();
        let role = fields.next().unwrap();
        for base in [0., -0., 0.25, -0.75, f32::NAN] {
            for general in (0..=54).chain([0x1002a, u32::MAX]) {
                for speed in [0., 0.375, 2.] {
                    let mut pose = NativeDpBodyPoseState::default();
                    pose.fields_3c[8] = selected;
                    pose.fields_3c[3] = general;
                    let mut actual_timer = Timer::default();
                    let actual = angle(&dna, &pose, base, speed, &mut actual_timer).unwrap();
                    let (offset, amplitude, unit_speed) = expected(role, base, general);
                    let mut expected_timer = Timer::default();
                    let expected = if let Some(amplitude) = amplitude {
                        let wave = native_dh_pose_rhythm(
                            &dna,
                            selected,
                            0.,
                            if unit_speed { 1. } else { speed },
                            &mut expected_timer,
                        )
                        .unwrap();
                        offset + wave * amplitude
                    } else {
                        offset
                    };
                    assert!(
                        actual.to_bits() == expected.to_bits()
                            || actual.is_nan() && expected.is_nan(),
                        "pose {selected}, role {role}, base {base}, general {general}, speed {speed}: {actual} != {expected}"
                    );
                    assert_eq!(actual_timer.events, expected_timer.events);
                }
            }
        }
    }
}

#[test]
fn full_width_defaults_never_sample_and_rhythm_failures_propagate() {
    let dna = NativeDhDna([0; 24]);
    let mut timer = Timer {
        fail: true,
        ..Default::default()
    };
    let mut pose = NativeDpBodyPoseState::default();
    for selected in [0, 1, 2, 4, 18, 83, 160, 256, 0x10011, u32::MAX] {
        pose.fields_3c[8] = selected;
        assert_eq!(
            angle(&dna, &pose, f32::NAN, 1., &mut timer)
                .unwrap()
                .to_bits(),
            0
        );
    }
    assert!(timer.events.is_empty());
    pose.fields_3c[8] = 17;
    assert_eq!(
        angle(&dna, &pose, 0., 1., &mut timer).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(timer.events, ["start"]);
}
