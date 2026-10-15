use super::*;

fn tag(kind: Kind) -> u8 {
    match kind {
        Kind::Default => 0,
        Kind::Constant(x) if x.to_bits() == 0x3fd55555 => 1,
        Kind::Standing(1.) => 2,
        Kind::RelativeStanding => 3,
        Kind::HalfWidth => 4,
        Kind::Standing(0.5) => 5,
        Kind::Neck => 6,
        Kind::Constant(0.) => 7,
        Kind::Constant(1.5) => 8,
        Kind::Constant(5.) => 9,
        _ => panic!("unexpected height kind"),
    }
}

#[test]
fn every_pose_matches_sha_bound_native_switch_fixture_without_byte_truncation() {
    let kinds: Vec<u8> = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/native_dp_height_kinds.tsv"
    ))
    .lines()
    .filter(|line| !line.starts_with('#'))
    .map(|line| line.split_whitespace().nth(1).unwrap().parse().unwrap())
    .collect();
    assert_eq!(kinds.len(), 138);
    for pose in 0..=255 {
        let expected = if pose < 138 { kinds[pose as usize] } else { 0 };
        assert_eq!(tag(kind(pose)), expected, "pose {pose}");
    }
    for pose in [256, 0x100 + 13, 0x8000_0000, u32::MAX] {
        assert_eq!(kind(pose), Kind::Default);
    }
}

#[test]
fn pose_height_requests_standing_only_for_legged_standing_classes() {
    for legs in [0, 4] {
        for class_bit in [0, 8] {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[12] = legs;
            dna.0[1] = class_bit;
            let dimensions = NativeDpDimensions::from_dna(&dna);
            let d = dimensions.fields_4080_through_40b8();
            let half = d[11] * 0.5;
            for pose in 0..=255 {
                for include in [false, true] {
                    let mut calls = vec![];
                    let actual = calculate(&dna, &dimensions, pose, include, |value| {
                        calls.push(value);
                        Ok(10.)
                    })
                    .unwrap();
                    let standing = matches!(kind(pose), Kind::Standing(_) | Kind::RelativeStanding);
                    assert_eq!(
                        calls,
                        if legs != 0 && standing {
                            vec![include]
                        } else {
                            vec![]
                        },
                        "pose {pose}"
                    );
                    let expected = match kind(pose) {
                        Kind::Constant(x) => x,
                        Kind::HalfWidth => d[12] * 0.5,
                        Kind::Standing(factor) => {
                            half.max(if legs == 0 { half } else { 10. } * factor)
                        }
                        Kind::RelativeStanding => half.max(
                            if legs == 0 { half } else { 10. }
                                * if class_bit == 0 {
                                    1.5
                                } else {
                                    f32::from_bits(0x3f2aaaab)
                                },
                        ),
                        Kind::Default | Kind::Neck => half,
                    };
                    assert_eq!(actual.to_bits(), expected.to_bits(), "pose {pose}");
                }
            }
        }
    }
}

#[test]
fn required_standing_failures_propagate_but_fixed_pose_paths_do_not_call_it() {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[12] = 4;
    let dimensions = NativeDpDimensions::from_dna(&dna);
    for pose in [13, 16, 17, 28, 75] {
        assert_eq!(
            calculate(&dna, &dimensions, pose, true, |_| Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "mounted geometry"
            )))
            .unwrap_err()
            .kind(),
            io::ErrorKind::Unsupported
        );
    }
    for pose in [0, 3, 19, 40, 55, 76, 80, 137, u32::MAX] {
        assert!(calculate(&dna, &dimensions, pose, true, |_| panic!()).is_ok());
    }
}

#[test]
fn neck_branches_preserve_lookup_boundaries_and_scalar_rounding() {
    for raw5 in 0..=255 {
        for count in 0..4 {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[5] = raw5;
            dna.0[14] = count;
            let dimensions = NativeDpDimensions::from_dna(&dna);
            let actual = calculate(&dna, &dimensions, 40, false, |_| panic!()).unwrap();
            assert_eq!(
                actual.to_bits(),
                (dimensions.fields_4080_through_40b8()[11] * 0.5).to_bits()
            );
        }
    }
    assert_eq!(maxss(0., -0.).to_bits(), (-0_f32).to_bits());
    assert!(maxss(1., f32::NAN).is_nan());
}
