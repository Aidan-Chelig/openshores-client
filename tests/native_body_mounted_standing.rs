//! Real mounted measurements plus composed leg bones; not a full-body preview.
use openshores_client::{
    asset_mount::AssetMount,
    world::{
        native_body::{
            NativeDpBoneTransforms, NativeDpFootCache, NativeDpHandCache, NativeDpInitialization,
            NativeDpPartKind,
        },
        native_dna::NativeDhDna,
    },
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; no original executable or GPU"]
fn mounted_standing_legs_compose_measured_angles_for_two_anatomies() {
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    let mut feet = NativeDpFootCache::new(&mount);
    let mut hands = NativeDpHandCache::new(&mount);
    for second in [false, true] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 1;
        dna.0[12] = if second { 8 } else { 4 };
        dna.0[14] = 0x51;
        dna.0[20] = if second { 0x80 } else { 0x60 };
        if second {
            dna.0[13] = 0x60;
            dna.0[15] = 0x40;
        }
        let mut body = NativeDpInitialization::begin(&dna);
        body.finish_part_calculations(0, &mut hands).unwrap();
        let height = feet
            .standing_height(&dna, body.dimensions(), 0, false)
            .unwrap();
        assert!(height.is_finite() && height > 0.);
        let before = body.bones().clone();
        let mut expected = body.clone();
        expected
            .spread_legs(f32::from_bits(0x3eb2b8c2), 2, 0, &mut feet)
            .unwrap();
        let mut expected_bones = before.clone();
        let legs: Vec<_> = expected
            .parts()
            .iter()
            .rev()
            .filter(|p| {
                matches!(
                    p.allocation.kind,
                    NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot
                )
            })
            .collect();
        assert_eq!(legs.len(), if second { 4 } else { 2 });
        for part in &legs {
            let angles = feet
                .leg_stand_angles(&dna, &part.allocation.dna, body.dimensions(), 0, height)
                .unwrap();
            part.stand_leg(&mut expected_bones, angles).unwrap();
        }
        body.stand_legs(height, 2, 0, &mut feet).unwrap();
        assert_eq!(body.bones(), &expected_bones);
        for part in legs {
            let indices = part.limb_bones().unwrap();
            for index in indices {
                assert_ne!(body.bones().slots[index], before.slots[index]);
                assert!(
                    body.bones().slots[index]
                        .matrix_00
                        .iter()
                        .all(|v| v.is_finite())
                );
            }
            let pivot = part.current_connect_loc(body.bones()).unwrap();
            for (a, b) in pivot.into_iter().zip(part.connect_30) {
                assert!((a - b).abs() < 1e-10);
            }
        }
        assert_eq!(&body.bones().slots[..41], &before.slots[..41]);
    }
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; measured angles only, no GPU"]
fn mounted_arm_angles_use_owner_hand_measurement_and_leave_bones_untouched() {
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    let mut hands = NativeDpHandCache::new(&mount);
    for wrist in [false, true] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 3;
        dna.0[11] = if wrist { 0 } else { 0x80 };
        let mut body = NativeDpInitialization::begin(&dna);
        body.finish_part_calculations(0, &mut hands).unwrap();
        let d = body.dimensions().fields_4080_through_40b8();
        let measured = hands.hand_length(&dna, 0, false).unwrap();
        let lower = if wrist { d[2] } else { measured + d[2] };
        let upper = d[13];
        let reach = (lower + upper) * f32::from_bits(0x3f733333);
        let denominator = -2. * upper;
        let shoulder =
            (((lower * lower - upper * upper) - reach * reach) / (denominator * reach)).acos();
        let elbow =
            (((reach * reach - upper * upper) - lower * lower) / (denominator * lower)).acos();
        let owner = NativeDpBoneTransforms::initialized();
        let before = owner.clone();
        let mut count = 0;
        for part in body.parts().iter().filter(|p| {
            matches!(
                p.allocation.kind,
                NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
            )
        }) {
            let angles = part
                .arm_stand_angles(&dna, body.dimensions(), &owner, 0, 100., &mut hands)
                .unwrap();
            assert_eq!(angles.shoulder, -f32::from_bits(0x3fc90fdb) - shoulder);
            assert_eq!(angles.elbow, f32::from_bits(0x40490fdb) - elbow);
            assert_eq!(angles.wrist, f32::from_bits(0x3d0efa35));
            let target = reach * 0.8;
            let expected = part
                .arm_stand_angles(&dna, body.dimensions(), &owner, 0, target, &mut hands)
                .unwrap();
            let mut translated = owner.clone();
            let first = part.limb_bones().unwrap()[0];
            // Origin W is the sum of all four fourth-row terms: 1+0+0+1.
            // Only this part's first-reference bone supplies the offset.
            translated.slots[first].matrix_00[3] = 1.;
            translated.slots[first].matrix_00[14] = 0.25;
            let actual = part
                .arm_stand_angles(
                    &dna,
                    body.dimensions(),
                    &translated,
                    0,
                    target - 0.125,
                    &mut hands,
                )
                .unwrap();
            for (a, b) in [
                (actual.shoulder, expected.shoulder),
                (actual.elbow, expected.elbow),
                (actual.wrist, expected.wrist),
            ] {
                assert!((a - b).abs() < 1e-6);
            }
            count += 1;
        }
        assert_eq!(count, 6);
        assert_eq!(owner, before);
    }
}
