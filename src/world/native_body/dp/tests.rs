use super::*;

#[test]
fn allocation_order_pair_indices_and_optional_parts() {
    use NativeDpPartKind::*;
    let mut dna = NativeDhDna([0; 24]);
    let minimal = native_allocate_dp_parts(&dna);
    assert_eq!(
        minimal.iter().map(|part| part.kind).collect::<Vec<_>>(),
        [Torso, HeadAndNeck]
    );
    dna.0[14] = 0x30;
    dna.0[5] = 2;
    dna.0[1] = 3;
    dna.0[12] = 12;
    let parts = native_allocate_dp_parts(&dna);
    assert_eq!(
        parts
            .iter()
            .map(|part| (part.kind, part.index))
            .collect::<Vec<_>>(),
        [
            (Torso, 0),
            (Tail, 0),
            (HeadAndNeck, 0),
            (HeadAndNeck, 1),
            (LeftArmAndHand, 0),
            (RightArmAndHand, 0),
            (LeftArmAndHand, 1),
            (RightArmAndHand, 1),
            (LeftArmAndHand, 2),
            (RightArmAndHand, 2),
            (LeftLegAndFoot, 0),
            (RightLegAndFoot, 0),
            (LeftLegAndFoot, 1),
            (RightLegAndFoot, 1),
            (LeftLegAndFoot, 2),
            (RightLegAndFoot, 2),
        ]
    );
    assert!(parts.iter().all(|part| part.dna == dna));
}

#[test]
fn all_arm_configuration_bytes_preserve_last_pair_and_original_dna() {
    for count_byte in 0..=255 {
        for wing_byte in 0..=255 {
            let mut dna = NativeDhDna([0xa5; 24]);
            dna.0[1] = count_byte;
            dna.0[2] = wing_byte;
            let original = dna.clone();
            let parts = native_allocate_dp_parts(&dna);
            let arms = parts
                .iter()
                .filter(|part| {
                    matches!(
                        part.kind,
                        NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand
                    )
                })
                .collect::<Vec<_>>();
            let pairs = count_byte & 3;
            assert_eq!(arms.len(), usize::from(pairs) * 2);
            for arm in arms {
                let mut expected = original.clone();
                if pairs > 1 && wing_byte & 6 != 0 && arm.index + 1 < u32::from(pairs) {
                    expected.0[2] &= 0xf8;
                }
                assert_eq!(arm.dna, expected);
            }
            assert_eq!(dna, original);
        }
    }
}

#[test]
fn optional_counts_ignore_other_bits_in_each_byte() {
    for raw in 0..=255 {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[5] = raw;
        dna.0[12] = raw;
        dna.0[14] = raw;
        let parts = native_allocate_dp_parts(&dna);
        assert_eq!(
            parts.len(),
            1 + usize::from(raw & 0x30 != 0)
                + usize::from(1 + ((raw >> 1) & 1))
                + usize::from((raw >> 2) & 3) * 2
        );
    }
}
