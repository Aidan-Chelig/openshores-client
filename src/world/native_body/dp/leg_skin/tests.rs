use super::*;

fn part(left: bool, index: u32) -> NativeDpPartState {
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: if left {
                NativeDpPartKind::LeftLegAndFoot
            } else {
                NativeDpPartKind::RightLegAndFoot
            },
            index,
            dna: NativeDhDna([0; 24]),
        },
        connect_30: [4., 5., 6.],
        joints: NativeDpPartJoints::Leg {
            field_50: [0.; 3],
            field_68: [0.; 3],
        },
    }
}

#[test]
fn sides_pairs_leg_variants_and_foot_ankle_selectors_match_native() {
    let mut owner = NativeDhDna([0; 24]);
    owner.0[16] = 5;
    let dimensions = NativeDpDimensions::from_dna(&owner);
    for left in [false, true] {
        for pair in 0..3 {
            for leg in 0..4 {
                for foot in 0..8 {
                    for field in [0, 0x41, 0x50, 0x99, 0x9b, 0x141] {
                        let mut part = part(left, pair);
                        part.allocation.dna.0[12] = leg << 4;
                        part.allocation.dna.0[10] = foot << 5;
                        let actual = part
                            .leg_skin_geometry(&owner, &dimensions, field, [1., 2., 3., 4.])
                            .unwrap();
                        let side = if left { "Left" } else { "Right" };
                        let pa = matches!(field, 0x41 | 0x50 | 0x99 | 0x9b);
                        let prefix = if pa { ":data/PA" } else { ":data/5" };
                        assert_eq!(actual.len(), 2);
                        assert_eq!(
                            actual[0].resource,
                            if pa {
                                format!("{prefix}Leg{side}.3dsN")
                            } else {
                                format!("{prefix}Leg{side}{leg}.3dsN")
                            }
                        );
                        let selector = if pa {
                            u8::from(native_dh_has_ankle_bend(&part.allocation.dna))
                        } else {
                            foot
                        };
                        assert_eq!(
                            actual[1].resource,
                            format!("{prefix}Foot{side}{selector}.3dsN")
                        );
                        assert_eq!(actual[0].bones, part.limb_bones().unwrap());
                        assert_eq!(actual[1].bones, actual[0].bones);
                    }
                }
            }
        }
    }
}

#[test]
fn leg_and_foot_definition_fields_keep_native_precision_and_replacement() {
    let owner = NativeDhDna([0; 24]);
    let dimensions = NativeDpDimensions::from_dna(&owner);
    let mut part = part(false, 0);
    part.allocation.dna.0[15] = 0xf8;
    part.allocation.dna.0[16] = 0x78;
    part.allocation.dna.0[2] = 0x38;
    part.allocation.dna.0[3] = 0x98;
    part.allocation.dna.0[10] = 0xff;
    let actual = part
        .leg_skin_geometry(&owner, &dimensions, 0, [1., 2., 3., 4.])
        .unwrap();
    let height = f64::from(31_f32 * 0.0625 + 0.5) / 1.5;
    let width = f64::from((15_f32 * 0.0625 + 1.) * 0.25);
    let leg_height = f64::from(7_f32 * 0.0625 + 0.5) / 1.5;
    let leg_width = f64::from((19_f32 * 0.0625 + 1.) * 0.25);
    assert_eq!(
        actual[0].geometry.fields_58,
        [
            height,
            (width + width) * height,
            leg_height,
            (leg_width + leg_width) * leg_height,
            1.
        ]
    );
    assert_eq!(
        actual[0].geometry.fields_00,
        [
            1.,
            f64::from(native_dp_size_factor(&owner) / 0.84_f32),
            1.,
            2.,
            3.,
            4.
        ]
    );
    let mut expected = actual[0].geometry;
    let d = dimensions.fields_4080_through_40b8();
    expected.fields_40[1] = (f64::from(d[7]) + 5.) + f64::from(d[0]);
    expected.fields_58[2] = f64::from(((31_f32 * 0.0625 + 0.5) * 0.25) * 2.);
    assert_eq!(actual[1].geometry, expected);
}

#[test]
fn invalid_leg_index_and_non_leg_or_incomplete_owner_fail_explicitly() {
    let owner = NativeDhDna([0; 24]);
    let dimensions = NativeDpDimensions::from_dna(&owner);
    assert_eq!(
        part(false, 3)
            .leg_skin_geometry(&owner, &dimensions, 0, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    let mut wrong = part(false, 0);
    wrong.allocation.kind = NativeDpPartKind::Torso;
    assert_eq!(
        wrong
            .leg_skin_geometry(&owner, &dimensions, 0, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(
        NativeDpInitialization::begin(&owner)
            .leg_skin_geometry_requests(0, 0, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
}
