use super::*;

fn leg(left: bool, pair: u32) -> NativeDpPartState {
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: if left {
                NativeDpPartKind::LeftLegAndFoot
            } else {
                NativeDpPartKind::RightLegAndFoot
            },
            index: pair,
            dna: NativeDhDna([0; 24]),
        },
        connect_30: [1., 2., 3.],
        joints: NativeDpPartJoints::Leg {
            splay_48: 0.25,
            field_50: [1., 4., 3.],
            field_68: [1., 6., 3.],
        },
    }
}

fn near(a: [f64; 3], b: [f64; 3]) {
    for i in 0..3 {
        assert!((a[i] - b[i]).abs() < 1e-12, "{a:?} != {b:?}");
    }
}

#[test]
fn leg_rotations_keep_actual_pivots_and_update_only_native_output_bones() {
    for left in [false, true] {
        for pair in 0..3 {
            for joint in 0..3 {
                let part = leg(left, pair);
                let indices = part.limb_bones().unwrap();
                let mut bones = NativeDpBoneTransforms::initialized();
                for bone in &mut bones.slots {
                    bone.translate([10., 20., 30.]);
                }
                let before = bones.clone();
                let pivot = match joint {
                    0 => part.connect_30,
                    1 => [1., 4., 3.],
                    _ => [1., 6., 3.],
                };
                let old_pivot = bones.slots[indices[joint]].project(pivot);
                match joint {
                    0 => part.rotate_leg_hip(&mut bones, 0.5),
                    1 => part.rotate_leg_knee(&mut bones, 0.5),
                    _ => part.rotate_leg_ankle(&mut bones, 0.5),
                }
                .unwrap();
                for i in 0..61 {
                    if indices[joint..].contains(&i) {
                        assert_ne!(bones.slots[i], before.slots[i]);
                        near(bones.slots[i].project(pivot), old_pivot);
                        let mut point = pivot;
                        point[1] += 1.;
                        let (s, c) = f64::from(0.5_f32).sin_cos();
                        near(
                            bones.slots[i].project(point),
                            [old_pivot[0], old_pivot[1] + c, old_pivot[2] + s],
                        );
                    } else {
                        assert_eq!(bones.slots[i], before.slots[i]);
                    }
                }
            }
        }
    }
}

#[test]
fn splay_mirrors_side_and_uses_current_not_rest_attachment() {
    for left in [false, true] {
        let part = leg(left, 1);
        let indices = part.limb_bones().unwrap();
        let mut bones = NativeDpBoneTransforms::initialized();
        for &i in &indices {
            bones.slots[i].translate([10., 20., 30.]);
        }
        let pivot = part.current_connect_loc(&bones).unwrap();
        part.splay_leg_hip(&mut bones, 0.5).unwrap();
        let angle = f64::from(if left { 0.5_f32 } else { -0.5 });
        let (s, c) = angle.sin_cos();
        for i in indices {
            near(bones.slots[i].project(part.connect_30), pivot);
            near(
                bones.slots[i].project([2., 2., 3.]),
                [pivot[0] + c, pivot[1] + s, pivot[2]],
            );
        }
    }
}

#[test]
fn stand_composes_rotations_in_native_order_and_honors_ankle_selector() {
    for ankle in [false, true] {
        let mut part = leg(true, 0);
        part.allocation.dna.0[10] = if ankle { 0 } else { 0x80 };
        let angles = NativeDpLegStandAngles {
            hip: 0.4,
            knee: -1.2,
            ankle: 0.9,
        };
        let mut actual = NativeDpBoneTransforms::initialized();
        let mut expected = actual.clone();
        part.rotate_leg_hip(&mut expected, angles.hip).unwrap();
        part.rotate_leg_knee(&mut expected, angles.knee).unwrap();
        if ankle {
            part.rotate_leg_ankle(&mut expected, angles.ankle).unwrap();
        }
        part.splay_leg_hip(&mut expected, 0.25).unwrap();
        part.stand_leg(&mut actual, angles).unwrap();
        assert_eq!(actual, expected);
        let mut wrong = NativeDpBoneTransforms::initialized();
        part.rotate_leg_knee(&mut wrong, angles.knee).unwrap();
        part.rotate_leg_hip(&mut wrong, angles.hip).unwrap();
        if ankle {
            part.rotate_leg_ankle(&mut wrong, angles.ankle).unwrap();
        }
        part.splay_leg_hip(&mut wrong, 0.25).unwrap();
        assert_ne!(actual, wrong);
    }
}

#[test]
fn zero_angles_skip_unwritten_bones_but_active_invalid_pairs_fail() {
    let mut part = leg(true, 9);
    let mut bones = NativeDpBoneTransforms::initialized();
    let before = bones.clone();
    part.rotate_leg_hip(&mut bones, -0.).unwrap();
    part.rotate_leg_knee(&mut bones, 0.).unwrap();
    part.rotate_leg_ankle(&mut bones, 0.).unwrap();
    part.splay_leg_hip(&mut bones, 0.).unwrap();
    assert_eq!(bones, before);
    assert_eq!(
        part.rotate_leg_hip(&mut bones, 1.).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    part.allocation.kind = NativeDpPartKind::Torso;
    assert_eq!(
        part.rotate_leg_knee(&mut bones, 0.).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
}
