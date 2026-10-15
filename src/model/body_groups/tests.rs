use super::*;
use crate::model::GroupVertex;

fn group(name: &[u8], count: usize, name_first: bool) -> VertexGroup {
    VertexGroup {
        name: Some(name.to_vec()),
        members: Some(
            (0..count)
                .map(|i| GroupVertex {
                    position: [i as f32, 1., 2.],
                    weight: i as f32 * 0.25,
                })
                .collect(),
        ),
        field_order: if name_first {
            vec![VertexGroupChunk::Name, VertexGroupChunk::Members]
        } else {
            vec![VertexGroupChunk::Members, VertexGroupChunk::Name]
        },
    }
}

#[test]
fn body_name_precedence_is_case_sensitive_and_does_not_invent_scale_subtypes() {
    for (name, kind) in [
        (b"ScalePt".as_slice(), 2),
        (b"TopScalePt", 2),
        (b"ScalePtConnectPt", 2),
        (b"ConnectPtHand", 3),
        (b"MidPtTorso", 4),
        (b"Top", 5),
        (b"Bottom", 6),
        (b"BottomSkinHand", 7),
        (b"TopConnect", 8),
        (b"BottomConnect", 9),
        (b"HandWingPt", 10),
        (b"EyePosition", 11),
        (b"scalept", 1),
    ] {
        let parsed = NativeBodyVertexGroup::from_group(&group(name, 0, true)).unwrap();
        assert_eq!(parsed.kind_34, kind);
        assert_eq!(parsed.flags_50, [false; 6]);
    }
    for (i, name) in [
        b"Torso".as_slice(),
        b"Hand",
        b"Foot",
        b"Neck",
        b"Tail",
        b"Head",
    ]
    .into_iter()
    .enumerate()
    {
        let parsed = NativeBodyVertexGroup::from_group(&group(name, 0, true)).unwrap();
        assert_eq!(parsed.kind_34, 1);
        assert_eq!(parsed.flags_50, std::array::from_fn(|j| i == j));
    }
    assert_eq!(
        classify(b"HandTorso").1,
        [true, false, false, false, false, false]
    );
}

#[test]
fn complete_special_name_types_keep_native_spellings() {
    for (name, kind) in [
        (b"SeatLoc".as_slice(), 15),
        (b"OffLoad", 16),
        (b"Offload", 16),
        (b"WeaponLoc", 17),
        (b"DirLoc", 18),
        (b"WeaponUDPt", 19),
        (b"WeaponLRPt", 20),
        (b"Screen1", 21),
        (b"Screen2", 22),
        (b"Flag Down", 23),
        (b"Flag Up", 24),
        (b"Missle Control", 25),
        (b"MissleDeploy", 26),
        (b"Seat1", 27),
        (b"Seat2", 28),
        (b"Seat3", 29),
        (b"Stand1", 30),
        (b"Stand2", 31),
        (b"Stand3", 32),
        (b"Stand4", 33),
        (b"Turret1", 34),
        (b"Turret2", 35),
        (b"Turret3", 36),
        (b"Turret4", 37),
    ] {
        assert_eq!(classify(name), (kind, [false; 6]), "{name:?}");
    }
    assert_eq!(classify(b"MissleControl").0, 1);
}

#[test]
fn one_record_never_becomes_membership_and_uses_only_the_fast_path_point_types() {
    for name in [
        b"Hand".as_slice(),
        b"ScalePt",
        b"ConnectPt",
        b"MidPt",
        b"Top",
        b"Bottom",
        b"BottomSkin",
        b"TopConnect",
        b"BottomConnect",
        b"HandWingPt",
        b"EyePosition",
        b"SeatLoc",
        b"Screen1",
        b"FlagDown",
        b"Turret1",
        b"unknown",
    ] {
        for name_first in [false, true] {
            let raw = group(name, 1, name_first);
            let parsed = NativeBodyVertexGroup::from_group(&raw).unwrap();
            assert!(parsed.entries().next().is_none());
            let kind = if name_first { classify(name).0 } else { 1 };
            let captures = matches!(kind, 3 | 10 | 11..=18 | 23 | 24 | 27..=33);
            assert_eq!(
                parsed.point_28,
                if captures { [0., 1., 2.] } else { [0.; 3] },
                "{name:?}"
            );
        }
    }
}

#[test]
fn multi_record_last_point_depends_on_classification_at_members_read_time() {
    for name in [
        b"Hand".as_slice(),
        b"Foot",
        b"Head",
        b"Torso",
        b"ScalePt",
        b"ConnectPt",
        b"MidPt",
        b"Top",
        b"Bottom",
        b"HandWingPt",
        b"EyePosition",
        b"BottomSkin",
        b"TopConnect",
        b"Screen1",
        b"unknown",
    ] {
        for count in [0, 2, 3, 12] {
            for name_first in [false, true] {
                let raw = group(name, count, name_first);
                let parsed = NativeBodyVertexGroup::from_group(&raw).unwrap();
                let (kind, flags) = if name_first {
                    classify(name)
                } else {
                    (1, [false; 6])
                };
                let point =
                    count > 0 && (matches!(kind, 2..=6 | 10 | 11) || flags.iter().any(|v| *v));
                assert_eq!(parsed.entries().count(), count - usize::from(point));
                assert_eq!(
                    parsed.point_28,
                    if point {
                        [(count - 1) as f32, 1., 2.]
                    } else {
                        [0.; 3]
                    }
                );
                if point {
                    assert_eq!(
                        parsed
                            .matching_weight([(count - 1) as f64, 1., 2.])
                            .unwrap(),
                        None
                    );
                }
            }
        }
    }
}

#[test]
fn duplicate_zero_keys_replace_weights_preserve_first_key_bits_and_sort_numerically() {
    let mut raw = group(b"unknown", 0, true);
    raw.members = Some(vec![
        GroupVertex {
            position: [-0., 1., 2.],
            weight: 0.25,
        },
        GroupVertex {
            position: [0., 1., 2.],
            weight: 0.,
        },
        GroupVertex {
            position: [-2., 1., 2.],
            weight: -0.5,
        },
        GroupVertex {
            position: [3., 1., 2.],
            weight: 0.75,
        },
    ]);
    let parsed = NativeBodyVertexGroup::from_group(&raw).unwrap();
    assert_eq!(parsed.entries().count(), 3);
    assert_eq!(parsed.matching_weight([0., 1., 2.]).unwrap(), Some(0.));
    let entries: Vec<_> = parsed.entries().collect();
    assert_eq!(entries[0].0[0], -2.);
    assert_eq!(entries[1].0[0].to_bits(), (-0_f32).to_bits());
    assert_eq!(entries[2].0[0], 3.);
    assert_eq!(raw.members.as_ref().unwrap().len(), 4);
}

#[test]
fn lookup_narrows_to_float_and_rejects_unordered_nan_keys_without_guessing() {
    let parsed = NativeBodyVertexGroup::from_group(&group(b"unknown", 2, true)).unwrap();
    assert_eq!(
        parsed.matching_weight([1. + 1e-9, 1., 2.]).unwrap(),
        Some(0.25)
    );
    assert_eq!(parsed.matching_weight([1. + 1e-6, 1., 2.]).unwrap(), None);
    assert_eq!(parsed.matching_weight([f64::MAX, 1., 2.]).unwrap(), None);
    assert_eq!(
        parsed
            .matching_weight([f64::NAN, 1., 2.])
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn missing_order_duplicate_fields_and_nonfinite_manual_records_are_explicit_errors() {
    let mut raw = group(b"Hand", 2, true);
    raw.field_order.clear();
    assert_eq!(
        NativeBodyVertexGroup::from_group(&raw).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    raw.field_order = vec![
        VertexGroupChunk::Name,
        VertexGroupChunk::Members,
        VertexGroupChunk::Name,
    ];
    assert!(NativeBodyVertexGroup::from_group(&raw).is_err());
    raw.field_order.pop();
    raw.members.as_mut().unwrap()[0].weight = f32::NAN;
    assert!(NativeBodyVertexGroup::from_group(&raw).is_err());
}
