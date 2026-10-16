//! Real mounted measurements and composed limb bones; not a full-body preview.
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
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; cached-only Fold leg selector coverage"]
fn mounted_fold_legs_preserve_other_bones_and_restore_stand() {
    use openshores_client::world::native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer};
    struct NoTimer;
    impl NativePoseRhythmTimer for NoTimer {
        fn start(&mut self) -> std::io::Result<()> {
            panic!("fixed Fold leg pose requested rhythm")
        }
        fn msecs_since_reference(&mut self) -> std::io::Result<i64> {
            panic!("fixed Fold leg pose requested clock")
        }
    }
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    for pairs in [1, 2, 3] {
        for ankle in [0, 0x80] {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[1] = 1;
            dna.0[12] = pairs << 2;
            dna.0[10] = ankle;
            dna.0[14] = 0x51;
            dna.0[20] = 0x80;
            let mut hands = NativeDpHandCache::new(&mount);
            let mut feet = NativeDpFootCache::new(&mount);
            let mut body = NativeDpInitialization::begin(&dna);
            body.finish_part_calculations(0, &mut hands).unwrap();
            body.pose_neutral_standing(0, &mut hands, &mut feet)
                .unwrap();
            let initial = body.clone();
            let mut hands = hands.into_cached_only();
            let mut feet = feet.into_cached_only();
            for selector in [0, 1, 2] {
                let mut pose = NativeDpBodyPoseState {
                    fields_3c: [0x24; 10],
                    field_6c: false,
                };
                if selector != 1 {
                    pose.fields_3c[4] = 0x18;
                }
                if selector != 0 {
                    pose.fields_3c[0] = 0x18;
                }
                let selected: std::collections::BTreeSet<_> = initial
                    .parts()
                    .iter()
                    .filter(|part| {
                        matches!(
                            part.allocation.kind,
                            NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot
                        ) && (selector == 2
                            || (selector == 0 && part.allocation.index != 0)
                            || (selector == 1 && part.allocation.index == 0))
                    })
                    .flat_map(|part| part.limb_bones().unwrap())
                    .collect();
                body.pose_locomotion_channels(&pose, 0., 0, &mut hands, &mut feet, &mut NoTimer)
                    .unwrap();
                let folded = body.bones().clone();
                if !selected.is_empty() {
                    assert_ne!(&folded, initial.bones());
                }
                for (index, bone) in folded.slots.iter().enumerate() {
                    assert!(bone.matrix_00.iter().all(|v| v.is_finite()));
                    if !selected.contains(&index) {
                        assert_eq!(bone, &initial.bones().slots[index]);
                    }
                }
                body.pose_locomotion_channels(&pose, 0., 0, &mut hands, &mut feet, &mut NoTimer)
                    .unwrap();
                assert_eq!(body.bones(), &folded);
                body.pose_neutral_standing(0, &mut hands, &mut feet)
                    .unwrap();
                assert_eq!(body.bones(), initial.bones());
            }
        }
    }
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; cached-only multi-arm animation, no GPU"]
fn mounted_multi_arm_actions_preserve_unselected_bones_and_repeat_on_one_body() {
    use openshores_client::world::native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer};
    struct Timer {
        millis: i64,
        samples: usize,
        starts: usize,
    }
    impl NativePoseRhythmTimer for Timer {
        fn start(&mut self) -> std::io::Result<()> {
            self.starts += 1;
            Ok(())
        }
        fn msecs_since_reference(&mut self) -> std::io::Result<i64> {
            self.samples += 1;
            Ok(self.millis)
        }
    }
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    for pairs in [1, 2, 3] {
        for wrist in [0x60, 0x80] {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[1] = pairs;
            dna.0[11] = wrist;
            dna.0[12] = 4;
            dna.0[14] = 0x51;
            dna.0[20] = 0x80;
            let mut hands = NativeDpHandCache::new(&mount);
            let mut feet = NativeDpFootCache::new(&mount);
            let mut body = NativeDpInitialization::begin(&dna);
            body.finish_part_calculations(0, &mut hands).unwrap();
            body.pose_neutral_standing(0, &mut hands, &mut feet)
                .unwrap();
            let initial = body.clone();
            let mut hands = hands.into_cached_only();
            let mut feet = feet.into_cached_only();
            for code in [0x25, 0x26, 0x30, 0x1c] {
                for side in [0, 1, 2] {
                    let mut pose = NativeDpBodyPoseState {
                        fields_3c: [0x24; 10],
                        field_6c: false,
                    };
                    if side != 1 {
                        pose.fields_3c[2] = code;
                    }
                    if side != 0 {
                        pose.fields_3c[5] = code;
                    }
                    let selected: std::collections::BTreeSet<_> = initial
                        .parts()
                        .iter()
                        .filter(|part| {
                            (side != 1 && part.allocation.kind == NativeDpPartKind::LeftArmAndHand)
                                || (side != 0
                                    && part.allocation.kind == NativeDpPartKind::RightArmAndHand)
                        })
                        .flat_map(|part| part.limb_bones().unwrap())
                        .collect();
                    let mut samples = Vec::new();
                    for millis in [0, 125, 0] {
                        let mut timer = Timer {
                            millis,
                            starts: 0,
                            samples: 0,
                        };
                        body.pose_locomotion_channels(
                            &pose, 4., 0, &mut hands, &mut feet, &mut timer,
                        )
                        .unwrap();
                        let expected = usize::from(pairs) * if side == 2 { 2 } else { 1 };
                        assert_eq!(timer.starts, expected);
                        assert_eq!(timer.samples, expected);
                        for (index, bone) in body.bones().slots.iter().enumerate() {
                            assert!(bone.matrix_00.iter().all(|value| value.is_finite()));
                            if !selected.contains(&index) {
                                assert_eq!(
                                    bone,
                                    &initial.bones().slots[index],
                                    "unselected bone {index}, pairs={pairs}, side={side}, code={code:#x}"
                                );
                            }
                        }
                        samples.push(body.bones().clone());
                    }
                    assert_eq!(samples[0], samples[2], "repeat accumulated transforms");
                    assert_ne!(samples[0], samples[1], "animation phase did not change");
                    body.pose_neutral_standing(0, &mut hands, &mut feet)
                        .unwrap();
                    assert_eq!(body.bones(), initial.bones());
                }
            }
        }
    }
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; native first-selected jump arm override"]
fn mounted_jump_arm_override_changes_only_first_selected_arm() {
    use openshores_client::world::{
        native_body::NativeDpArmJump, native_creature::NativePoseRhythmTimer,
    };
    struct Timer;
    impl NativePoseRhythmTimer for Timer {
        fn start(&mut self) -> std::io::Result<()> {
            Ok(())
        }
        fn msecs_since_reference(&mut self) -> std::io::Result<i64> {
            Ok(125)
        }
    }
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    for wrist in [0x60, 0x80] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 3;
        dna.0[11] = wrist;
        dna.0[12] = 4;
        let mut hands = NativeDpHandCache::new(&mount);
        let mut feet = NativeDpFootCache::new(&mount);
        let mut base = NativeDpInitialization::begin(&dna);
        base.finish_part_calculations(0, &mut hands).unwrap();
        let height = feet
            .standing_height(&dna, base.dimensions(), 0, false)
            .unwrap();
        for side in [0, 1, 2] {
            let mut ordinary = base.clone();
            let mut overridden = base.clone();
            let mut input = NativeDpArmJump {
                height,
                speed: 4.,
                side,
                override_field_30: None,
                field_4068: 0,
            };
            ordinary.jump_arms(input, &mut hands, &mut Timer).unwrap();
            input.override_field_30 = Some(0.4);
            overridden.jump_arms(input, &mut hands, &mut Timer).unwrap();
            let first = base
                .parts()
                .iter()
                .find(|part| {
                    (side != 1 && part.allocation.kind == NativeDpPartKind::LeftArmAndHand)
                        || (side != 0 && part.allocation.kind == NativeDpPartKind::RightArmAndHand)
                })
                .unwrap()
                .limb_bones()
                .unwrap();
            assert!(
                first
                    .iter()
                    .any(|&index| ordinary.bones().slots[index] != overridden.bones().slots[index])
            );
            for (index, bone) in ordinary.bones().slots.iter().enumerate() {
                if !first.contains(&index) {
                    assert_eq!(
                        bone,
                        &overridden.bones().slots[index],
                        "override leaked beyond first selected arm"
                    );
                }
            }
        }
    }
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; static native locomotion port, no original runtime"]
fn mounted_walk_run_jump_leg_samples_change_selected_bones_without_accumulation() {
    use openshores_client::world::{
        native_body::{NativeDpGait, NativeDpLegLocomotion},
        native_creature::NativePoseRhythmTimer,
    };
    struct Timer {
        millis: i64,
        starts: usize,
        samples: usize,
    }
    impl NativePoseRhythmTimer for Timer {
        fn start(&mut self) -> std::io::Result<()> {
            self.starts += 1;
            Ok(())
        }
        fn msecs_since_reference(&mut self) -> std::io::Result<i64> {
            self.samples += 1;
            Ok(self.millis)
        }
    }
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    let mut hands = NativeDpHandCache::new(&mount);
    let mut feet = NativeDpFootCache::new(&mount);
    for (pairs, ankle) in [(1, 0), (1, 0x80), (2, 0), (2, 0x80), (3, 0), (3, 0x80)] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 1;
        dna.0[10] = ankle;
        dna.0[12] = pairs << 2;
        dna.0[14] = 0x51;
        dna.0[20] = 0x80;
        let mut base = NativeDpInitialization::begin(&dna);
        base.finish_part_calculations(0, &mut hands).unwrap();
        let height = feet
            .standing_height(&dna, base.dimensions(), 0, false)
            .unwrap();
        for gait in [Some(NativeDpGait::Walk), Some(NativeDpGait::Run), None] {
            for selector in [0, 1, 2] {
                let mut sampled = Vec::new();
                for millis in [0, 125, 0] {
                    let mut body = base.clone();
                    let mut timer = Timer {
                        millis,
                        starts: 0,
                        samples: 0,
                    };
                    if let Some(gait) = gait {
                        body.locomote_legs(
                            NativeDpLegLocomotion {
                                gait,
                                selector,
                                height,
                                speed: 4.,
                                field_4068: 0,
                            },
                            &mut feet,
                            &mut timer,
                        )
                        .unwrap();
                    } else {
                        body.jump_legs(height, 4., selector, 0, &mut feet, &mut timer)
                            .unwrap();
                    }
                    let mut selected_bones = std::collections::BTreeSet::new();
                    let mut count = 0;
                    for (before, part) in base.parts().iter().zip(body.parts()) {
                        let leg = matches!(
                            part.allocation.kind,
                            NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot
                        );
                        let selected = selector == 2
                            || (selector == 0 && part.allocation.index != 0)
                            || (selector == 1 && part.allocation.index == 0);
                        if leg && selected {
                            count += 1;
                            selected_bones.extend(part.limb_bones().unwrap());
                        } else {
                            assert_eq!(before, part);
                        }
                    }
                    assert_eq!(timer.starts, count);
                    assert_eq!(timer.samples, count);
                    for (index, bone) in body.bones().slots.iter().enumerate() {
                        assert!(bone.matrix_00.iter().all(|v| v.is_finite()));
                        if !selected_bones.contains(&index) {
                            assert_eq!(bone, &base.bones().slots[index]);
                        }
                    }
                    sampled.push(body.bones().clone());
                }
                assert_eq!(sampled[0], sampled[2]);
                if pairs > 1 || selector != 0 {
                    assert_ne!(sampled[0], sampled[1]);
                }
            }
        }
    }
}

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
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; full neutral CPU pose, no GPU"]
fn mounted_neutral_pose_composes_all_stages_and_repeats_without_accumulation() {
    use openshores_client::world::native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer};
    use std::io;
    struct NoTimer;
    impl NativePoseRhythmTimer for NoTimer {
        fn start(&mut self) -> io::Result<()> {
            panic!("Stand must not request a rhythm")
        }
        fn msecs_since_reference(&mut self) -> io::Result<i64> {
            panic!("Stand must not request a clock")
        }
    }
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    let mut hands = NativeDpHandCache::new(&mount);
    let mut feet = NativeDpFootCache::new(&mount);
    for second in [false, true] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = if second { 2 } else { 1 };
        dna.0[12] = if second { 8 } else { 4 };
        dna.0[14] = if second { 0x91 } else { 0x51 };
        dna.0[20] = if second { 0x80 } else { 0x60 };
        if second {
            dna.0[5] = 2;
            dna.0[13] = 0x60;
            dna.0[15] = 0x40;
        }
        let mut body = NativeDpInitialization::begin(&dna);
        body.finish_part_calculations(0, &mut hands).unwrap();
        let mut expected = body.clone();
        let pose = NativeDpBodyPoseState {
            fields_3c: [0x24; 10],
            field_6c: false,
        };
        let height = feet
            .pose_height(&dna, expected.dimensions(), 0, &pose, false)
            .unwrap();
        expected.stand_legs(height, 0, 0, &mut feet).unwrap();
        expected.stand_legs(height, 1, 0, &mut feet).unwrap();
        expected
            .position_tail(&pose, height, 0., false, &mut NoTimer)
            .unwrap();
        let torso = expected
            .position_torso(&pose, height, 0., 0, &mut hands, &mut NoTimer)
            .unwrap();
        expected
            .position_heads(&pose, 0., 0., &mut NoTimer)
            .unwrap();
        expected.stand_arms(height, 0, None, 0, &mut hands).unwrap();
        expected.stand_arms(height, 1, None, 0, &mut hands).unwrap();
        let result = body
            .pose_neutral_standing(0, &mut hands, &mut feet)
            .unwrap();
        assert_eq!(result.pose, pose);
        assert_eq!(result.height, height);
        assert_eq!(result.torso_angle, torso);
        assert_eq!(body.bones(), expected.bones());
        assert_eq!(body.parts(), expected.parts());
        assert_ne!(body.bones(), &NativeDpBoneTransforms::initialized());
        let first = body.bones().clone();
        let again = body
            .pose_neutral_standing(0, &mut hands, &mut feet)
            .unwrap();
        assert_eq!(result, again);
        assert_eq!(body.bones(), &first);
        for part in body.parts().iter().filter(|p| {
            matches!(
                p.allocation.kind,
                NativeDpPartKind::LeftArmAndHand
                    | NativeDpPartKind::RightArmAndHand
                    | NativeDpPartKind::LeftLegAndFoot
                    | NativeDpPartKind::RightLegAndFoot
            )
        }) {
            for index in part.limb_bones().unwrap() {
                assert_ne!(
                    body.bones().slots[index],
                    NativeDpBoneTransforms::initialized().slots[index]
                );
            }
        }
    }
    for (legs, wings) in [(0, 0), (4, 2)] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 1;
        dna.0[12] = legs;
        dna.0[2] = wings;
        let mut body = NativeDpInitialization::begin(&dna);
        body.finish_part_calculations(0, &mut hands).unwrap();
        let before = body.bones().clone();
        assert_eq!(
            body.pose_neutral_standing(0, &mut hands, &mut feet)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(body.bones(), &before);
    }
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; composed arm phase, no GPU"]
fn mounted_standing_arms_match_order_side_wings_and_first_selected_override() {
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    let mut hands = NativeDpHandCache::new(&mount);
    for walks_on_hands in [false, true] {
        for wings in [false, true] {
            for side in [-1, 0, 1, 2, 3] {
                for override_field in [None, Some(0.3)] {
                    let mut dna = NativeDhDna([0; 24]);
                    dna.0[1] = 3;
                    dna.0[11] = if walks_on_hands { 0 } else { 0x60 };
                    dna.0[2] = if wings { 2 } else { 0 };
                    let mut body = NativeDpInitialization::begin(&dna);
                    body.finish_part_calculations(0, &mut hands).unwrap();
                    let before = body.bones().clone();
                    let mut expected = body.clone();
                    let lift = if walks_on_hands {
                        f32::from_bits(0x3c8efa35)
                    } else {
                        f32::from_bits(0x3e567750)
                    };
                    expected
                        .spread_arms(f32::from_bits(0x3eb2b8c2), lift, side)
                        .unwrap();
                    let mut expected_bones = before.clone();
                    let mut pending_override = override_field;
                    let mut selected_count = 0;
                    for part in expected.parts().iter().skip(1) {
                        let left = part.allocation.kind == NativeDpPartKind::LeftArmAndHand;
                        let right = part.allocation.kind == NativeDpPartKind::RightArmAndHand;
                        let ordinary = (left || right)
                            && (part.allocation.dna.0[1] & 3 == 0
                                || part.allocation.dna.0[2] & 6 == 0);
                        let selected = side == 2 || (side == 0 && left) || (side == 1 && right);
                        if ordinary && selected {
                            let angles = part
                                .arm_stand_angles(
                                    &dna,
                                    expected.dimensions(),
                                    &expected_bones,
                                    0,
                                    100.,
                                    &mut hands,
                                )
                                .unwrap();
                            part.relax_arm(&mut expected_bones, angles, pending_override.take())
                                .unwrap();
                            selected_count += 1;
                        }
                    }
                    body.stand_arms(100., side, override_field, 0, &mut hands)
                        .unwrap();
                    assert_eq!(body.bones(), &expected_bones);
                    assert_eq!(body.parts(), expected.parts());
                    assert_eq!(
                        selected_count,
                        match side {
                            0 | 1 =>
                                if wings {
                                    2
                                } else {
                                    3
                                },
                            2 =>
                                if wings {
                                    4
                                } else {
                                    6
                                },
                            _ => 0,
                        }
                    );
                    assert_eq!(&body.bones().slots[..23], &before.slots[..23]);
                    assert_eq!(&body.bones().slots[41..], &before.slots[41..]);
                    for part in body.parts().iter().filter(|p| {
                        matches!(
                            p.allocation.kind,
                            NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
                        )
                    }) {
                        let indices = part.limb_bones().unwrap();
                        for index in indices {
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
                }
            }
        }
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
