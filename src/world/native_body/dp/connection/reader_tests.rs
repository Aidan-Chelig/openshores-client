use super::*;
use crate::world::{
    native_body::{NativeDpAllocatedPart, NativeDpPartJoints},
    native_dna::NativeDhDna,
};

fn part(kind: NativeDpPartKind, index: u32, dna: &NativeDhDna) -> NativeDpPartState {
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind,
            index,
            dna: dna.clone(),
        },
        connect_30: [1., 2., 3.],
        joints: NativeDpPartJoints::None,
    }
}

#[test]
fn limb_triplets_match_native_outputs_including_asymmetric_invalid_indices() {
    use NativeDpPartKind::*;
    for (kind, expected) in [
        (LeftArmAndHand, [[25, 24, 23], [31, 30, 29], [37, 36, 35]]),
        (RightArmAndHand, [[28, 27, 26], [34, 33, 32], [40, 39, 38]]),
        (LeftLegAndFoot, [[43, 42, 41], [49, 48, 47], [55, 54, 53]]),
        (RightLegAndFoot, [[46, 45, 44], [52, 51, 50], [58, 57, 56]]),
    ] {
        for index in [0, 1, 2, 3, 0x80000000, u32::MAX] {
            let part = part(kind, index, &NativeDhDna([0; 24]));
            if index > 2 && matches!(kind, LeftLegAndFoot | RightLegAndFoot) {
                assert_eq!(
                    part.limb_bones().unwrap_err().kind(),
                    io::ErrorKind::Unsupported
                );
            } else {
                assert_eq!(
                    part.limb_bones().unwrap(),
                    expected[if index > 2 { 0 } else { index as usize }]
                );
            }
        }
    }
}

#[test]
fn every_concrete_limb_projects_through_first_native_out_reference() {
    use NativeDpPartKind::*;
    let mut bones = NativeDpBoneTransforms::initialized();
    for (index, bone) in bones.slots.iter_mut().enumerate() {
        bone.translate([index as f64, 100. + index as f64, 200. + index as f64]);
    }
    for kind in [
        LeftArmAndHand,
        RightArmAndHand,
        LeftLegAndFoot,
        RightLegAndFoot,
    ] {
        for index in 0..3 {
            let part = part(kind, index, &NativeDhDna([0; 24]));
            let slot = part.limb_bones().unwrap()[0] as f64;
            assert_eq!(
                part.current_connect_loc(&bones).unwrap(),
                [slot + 1., slot + 102., slot + 203.]
            );
        }
    }
}

#[test]
fn head_and_torso_selectors_use_part_dna_and_not_limb_or_tail_bones() {
    use NativeDpPartKind::*;
    let mut bones = NativeDpBoneTransforms::initialized();
    for (index, bone) in bones.slots.iter_mut().enumerate() {
        bone.translate([index as f64, 10., 20.]);
    }
    for raw5 in 0..=255 {
        for count in 0..4 {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[5] = raw5;
            dna.0[14] = count;
            for index in [0, 1, 2, u32::MAX] {
                let p = part(HeadAndNeck, index, &dna);
                let slot =
                    7 + u32::from(count) + u32::from((raw5 >> 1) & 1) + u32::from(index == 1) * 5;
                assert_eq!(
                    p.current_connect_loc(&bones).unwrap(),
                    [f64::from(slot) + 1., 12., 23.]
                );
            }
        }
    }
    for last in 0..8 {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[20] = last << 5;
        dna.0[12] = 4;
        // One leg pair makes WaistVertex equal last (including zero).
        let p = part(Torso, 0, &dna);
        assert_eq!(
            p.current_connect_loc(&bones).unwrap(),
            [f64::from(last.saturating_sub(1)), 10., 20.]
        );
        assert_eq!(p.connect_30, [1., 2., 3.]); // Torso must ignore this value.
    }
}
