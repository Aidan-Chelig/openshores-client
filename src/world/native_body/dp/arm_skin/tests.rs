use super::*;

fn part(left: bool, index: u32, selector: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[2] = selector;
    dna.0[11] = 2 << 5;
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: if left {
                NativeDpPartKind::LeftArmAndHand
            } else {
                NativeDpPartKind::RightArmAndHand
            },
            index,
            dna,
        },
        connect_30: [4., 5., 6.],
        joints: NativeDpPartJoints::Arm {
            field_58: [0.; 3],
            field_70: [0.; 3],
            field_88: [0.; 3],
        },
    }
}

#[test]
fn all_sides_pairs_selectors_and_pa_modes_choose_native_ordered_resources() {
    let mut owner = NativeDhDna([0; 24]);
    owner.0[16] = 3;
    let dimensions = NativeDpDimensions::from_dna(&owner);
    for left in [false, true] {
        for index in [0, 1, 2, 3, u32::MAX] {
            for selector in 0..8 {
                for raw in [0, 0x41, 0x50, 0x99, 0x9b, 0x141] {
                    let part = part(left, index, selector);
                    let actual = part
                        .arm_skin_geometry(&owner, &dimensions, raw, [1., 2., 3., 4.])
                        .unwrap();
                    let side = if left { "Left" } else { "Right" };
                    let pa = matches!(raw, 0x41 | 0x50 | 0x99 | 0x9b);
                    let prefix = if pa { ":data/PA" } else { ":data/3" };
                    assert_eq!(actual.len(), if selector > 1 { 1 } else { 2 });
                    assert_eq!(
                        actual[0].resource,
                        if selector > 1 {
                            format!(":data/Wing{side}{selector}.3dsN")
                        } else if pa {
                            format!("{prefix}Arm{side}.3dsN")
                        } else {
                            format!("{prefix}Arm{side}{selector}.3dsN")
                        }
                    );
                    assert_eq!(actual[0].bones, part.limb_bones().unwrap());
                    if selector < 2 {
                        assert_eq!(
                            actual[1].resource,
                            if pa {
                                format!("{prefix}Hand{side}.3dsN")
                            } else {
                                format!("{prefix}Hand{side}2.3dsN")
                            }
                        );
                        assert_eq!(actual[1].bones, actual[0].bones);
                    }
                }
            }
        }
    }
}

#[test]
fn geometry_preserves_float_stages_owner_fields_and_hand_specific_replacement() {
    let owner = NativeDhDna([0; 24]);
    let dimensions = NativeDpDimensions::from_dna(&owner);
    let mut part = part(false, 0, 0);
    part.allocation.dna.0[6] = 0xf8;
    part.allocation.dna.0[8] = 0x78;
    part.allocation.dna.0[21] = 31;
    part.allocation.dna.0[22] = 17;
    part.allocation.dna.0[11] = 0xff;
    let actual = part
        .arm_skin_geometry(&owner, &dimensions, 0, [1., 2., 3., 4.])
        .unwrap();
    let a = &actual[0].geometry;
    let b = &actual[1].geometry;
    assert_eq!(
        a.fields_00,
        [
            1.,
            f64::from(native_dp_size_factor(&owner) / 0.84_f32),
            1.,
            2.,
            3.,
            4.
        ]
    );
    let torso = f64::from(31_f32 * 0.0625 + 0.5) / 1.5;
    let width = f64::from((17_f32 * 0.0625 + 1.) * 0.25);
    let arm = f64::from(31_f32 * 0.0625 + 0.5) / 1.5;
    let arm_width = f64::from((15_f32 * 0.0625 + 1.) * 0.25);
    assert_eq!(
        a.fields_58,
        [
            torso,
            (width + width) * torso,
            arm,
            (arm_width + arm_width) * arm,
            1.
        ]
    );
    assert_eq!(a.fields_40, [4., 5., 6.]);
    let d = dimensions.fields_4080_through_40b8();
    assert_eq!(
        b.fields_40,
        [4., (f64::from(d[13]) + 5.) + f64::from(d[2]), 6.]
    );
    assert_eq!(
        b.fields_58[2],
        f64::from(((31_f32 * 0.0625 + 0.5) * 0.25) * 2.)
    );
    assert_eq!(b.fields_58[4], 1.);
    part.allocation.dna.0[2] = 2;
    let wing = part
        .arm_skin_geometry(&owner, &dimensions, 0, [1., 2., 3., 4.])
        .unwrap();
    assert_eq!(wing[0].geometry.fields_58[4], b.fields_58[2]);
}

#[test]
fn wrong_part_and_incomplete_body_do_not_publish_requests() {
    let owner = NativeDhDna([0; 24]);
    let dimensions = NativeDpDimensions::from_dna(&owner);
    let mut part = part(false, 0, 0);
    part.allocation.kind = NativeDpPartKind::Torso;
    assert_eq!(
        part.arm_skin_geometry(&owner, &dimensions, 0, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(
        NativeDpInitialization::begin(&owner)
            .arm_skin_geometry_requests(0, 0, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
}
