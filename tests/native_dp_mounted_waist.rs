//! Actual mounted16-part body RotateWaist composition, entirely offline.
use openshores_client::{
    asset_mount::AssetMount,
    world::{
        native_body::{
            NativeDpHandCache, NativeDpInitialization, NativeDpPartKind, native_dh_waist_vertex,
        },
        native_dna::NativeDhDna,
    },
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn mounted_body_waist_moves_heads_and_arms_but_not_tail_or_legs() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut cache = NativeDpHandCache::new(&mount);
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 3;
    dna.0[2] = 2;
    dna.0[5] = 2;
    dna.0[12] = 12;
    dna.0[14] = 0x31;
    dna.0[20] = 0x60;
    let mut body = NativeDpInitialization::begin(&dna);
    body.finish_part_calculations(0, &mut cache).unwrap();
    assert_eq!(body.parts().len(), 16);
    assert_eq!(native_dh_waist_vertex(&dna), 2);
    for angle in [0.25_f32, -0.125, 0.375] {
        let before = body.bones().clone();
        let pivot = body.current_waist_loc().unwrap();
        let attachments: Vec<_> = body
            .parts()
            .iter()
            .map(|p| p.current_connect_loc(&before).unwrap())
            .collect();
        let (s, c) = f64::from(angle).sin_cos();
        assert!(body.rotate_waist(angle).unwrap());
        assert_eq!(body.current_waist_loc().unwrap(), pivot);
        for (part, old) in body.parts().iter().zip(attachments) {
            match part.allocation.kind {
                NativeDpPartKind::HeadAndNeck
                | NativeDpPartKind::LeftArmAndHand
                | NativeDpPartKind::RightArmAndHand => {
                    let local = [old[0] - pivot[0], old[1] - pivot[1], old[2] - pivot[2]];
                    let expected = [
                        pivot[0] + local[0],
                        pivot[1] + local[1] * c - local[2] * s,
                        pivot[2] + local[1] * s + local[2] * c,
                    ];
                    let actual = part.current_connect_loc(body.bones()).unwrap();
                    for axis in 0..3 {
                        assert!((actual[axis] - expected[axis]).abs() < 1e-12);
                    }
                    if part.allocation.kind != NativeDpPartKind::HeadAndNeck {
                        for index in part.limb_bones().unwrap() {
                            assert_eq!(
                                &body.bones().slots[index].matrix_00[..12],
                                &before.slots[index].matrix_00[..12]
                            );
                        }
                    }
                }
                NativeDpPartKind::Tail => assert_eq!(body.bones().slots[17], before.slots[17]),
                NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot => {
                    for index in part.limb_bones().unwrap() {
                        assert_eq!(body.bones().slots[index], before.slots[index]);
                    }
                }
                NativeDpPartKind::Torso => {}
            }
        }
    }
    let before = body.bones().clone();
    assert!(!body.rotate_waist(-0.).unwrap());
    assert_eq!(*body.bones(), before);

    // Shoulder pivots must use the CURRENT attachment after the preceding
    // body-wide rotations, not the stored bind-pose ConnectLoc.
    for part in body.parts().iter().filter(|p| {
        matches!(
            p.allocation.kind,
            NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
        )
    }) {
        let mut bones = body.bones().clone();
        let indices = part.limb_bones().unwrap();
        let raw = &part.allocation.dna.0;
        let all = raw[1] & 3 == 0 || matches!(raw[2] & 7, 0 | 1 | 5 | 6);
        let selected = &indices[..if all { 3 } else { 1 }];
        for angle in [0.25_f32, -0.125, 0.375] {
            let before = bones.clone();
            let pivot = part.current_connect_loc(&bones).unwrap();
            let (s, c) = f64::from(angle).sin_cos();
            part.rotate_arm_shoulder(&mut bones, angle).unwrap();
            let actual_pivot = part.current_connect_loc(&bones).unwrap();
            for axis in 0..3 {
                assert!((actual_pivot[axis] - pivot[axis]).abs() < 1e-12);
            }
            for (index, bone) in bones.slots.iter().enumerate() {
                if !selected.contains(&index) {
                    assert_eq!(*bone, before.slots[index]);
                    continue;
                }
                assert_eq!(bone.field_100, 0);
                let old = before.slots[index].project([0.125, -0.25, 0.5]);
                let local = [old[0] - pivot[0], old[1] - pivot[1], old[2] - pivot[2]];
                let expected = [
                    old[0],
                    pivot[1] + local[1] * c - local[2] * s,
                    pivot[2] + local[1] * s + local[2] * c,
                ];
                let actual = bone.project([0.125, -0.25, 0.5]);
                for axis in 0..3 {
                    assert!((actual[axis] - expected[axis]).abs() < 1e-12);
                }
            }
        }
    }
}
