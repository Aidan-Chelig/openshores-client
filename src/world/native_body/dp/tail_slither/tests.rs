use super::*;

fn tail(selector: u8) -> NativeDpPartState {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[14] = (selector << 6) | 0x30;
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: NativeDpPartKind::Tail,
            index: 0,
            dna,
        },
        connect_30: [0., 2., 0.],
        joints: NativeDpPartJoints::None,
    }
}

#[test]
fn zero_slither_still_writes_and_invalid_kinds_do_not() {
    let mut part = tail(3);
    let dims = NativeDpDimensions::from_dna(&part.allocation.dna);
    let mut bones = NativeDpBoneTransforms::initialized();
    for bone in &mut bones.slots {
        bone.field_100 = 199;
    }
    part.slither_tail(&mut bones, &dims, 0., -0.).unwrap();
    for (i, bone) in bones.slots.iter().enumerate() {
        assert_eq!(bone.field_100, if (17..=22).contains(&i) { 0 } else { 199 });
    }
    part.allocation.kind = NativeDpPartKind::Torso;
    let before = bones.clone();
    assert_eq!(
        part.slither_tail(&mut bones, &dims, 0.25, -0.125)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(
        part.tail_can_bend_laterally().unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(
        part.tail_can_bend_vertically().unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(bones, before);
    let mut body = NativeDpInitialization::begin(&part.allocation.dna);
    let before = body.bones().clone();
    assert_eq!(
        body.dead_tail_facing_down().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        body.dead_tail_facing_left().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(*body.bones(), before);
}

#[test]
fn capability_tables_cover_all_shape_form_bits() {
    // Rows: no tail, shape10/20/30. Columns: low three bits DNA16.
    let lateral = [
        [false; 8],
        [false, true, true, false, true, false, true, false],
        [true, false, false, false, false, true, false, false],
        [false, false, true, true, true, false, true, true],
    ];
    let vertical = [
        [false; 8],
        [true, true, true, true, true, true, false, true],
        [false, true, true, true, true, false, true, true],
        [true, true, true, true, true, true, true, false],
    ];
    for raw in 0..=255u8 {
        for form in 0..=255u8 {
            let mut part = tail(0);
            part.allocation.dna.0[14] = raw;
            part.allocation.dna.0[16] = form;
            let row = usize::from((raw >> 4) & 3);
            let col = usize::from(form & 7);
            assert_eq!(part.tail_can_bend_laterally().unwrap(), lateral[row][col]);
            assert_eq!(part.tail_can_bend_vertically().unwrap(), vertical[row][col]);
        }
    }
}

#[test]
fn slither_reverses_middle_and_halves_root_without_touching_neighbors() {
    // Independently accumulated joint angles for last17,18,20,22.
    let expected: [&[f64]; 4] = [
        &[0.5],
        &[0.5, 1.5],
        &[-0.5, -1.5, -0.5, 0.5],
        &[-0.5, -1.5, -2.5, -1.5, -0.5, 0.5],
    ];
    for selector in 0..4 {
        let part = tail(selector);
        let dims = NativeDpDimensions::from_dna(&part.allocation.dna);
        let mut bones = NativeDpBoneTransforms::initialized();
        for bone in &mut bones.slots {
            bone.field_100 = 199;
        }
        let before = bones.clone();
        part.slither_tail(&mut bones, &dims, 0.25, 0.).unwrap();
        let end = 16 + expected[usize::from(selector)].len();
        for (i, bone) in bones.slots.iter().enumerate() {
            if (17..=end).contains(&i) {
                let angle = expected[usize::from(selector)][i - 17] * 0.25;
                assert!(
                    (bone.matrix_00[5] - angle.cos()).abs() < 1e-12,
                    "selector {selector}, bone {i}"
                );
                assert!(
                    (bone.matrix_00[6] - angle.sin()).abs() < 1e-12,
                    "selector {selector}, bone {i}"
                );
                assert_eq!(bone.field_100, 0);
            } else {
                assert_eq!(*bone, before.slots[i]);
            }
        }
    }
}
