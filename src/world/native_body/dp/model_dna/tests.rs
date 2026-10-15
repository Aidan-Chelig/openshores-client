use super::*;

#[test]
fn all_fields_match_packed_maxima_and_last_field_is_retained() {
    let mut key = NativeDpModelDna::from_fields([u32::MAX; 48]);
    key.apply_dhdna(&NativeDhDna([255; 24]));
    assert_eq!(
        *key.fields(),
        [
            31,
            7,
            3,
            3,
            1,
            3,
            3,
            7,
            3,
            31,
            31,
            3,
            3,
            7,
            7,
            31,
            7,
            31,
            7,
            7,
            31,
            31,
            31,
            31,
            31,
            31,
            31,
            31,
            7,
            3,
            7,
            3,
            3,
            3,
            3,
            7,
            3,
            7,
            3,
            3,
            3,
            3,
            15,
            3,
            7,
            7,
            3,
            u32::MAX,
        ]
    );
    let default = NativeDpModelDna::from_dhdna(&NativeDhDna([0; 24]));
    assert_eq!(default.fields(), &[0; 48]);
}

#[test]
fn native_comparison_uses_every_unsigned_field_in_verified_priority() {
    let unique = ORDER.into_iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        unique.into_iter().collect::<Vec<_>>(),
        (0..48).map(|i| i * 4).collect::<Vec<_>>()
    );
    for (priority, offset) in ORDER.into_iter().enumerate() {
        let mut low = [0; 48];
        let mut high = [0; 48];
        low[offset / 4] = 0x7fff_ffff;
        high[offset / 4] = 0x8000_0000;
        for later in &ORDER[priority + 1..] {
            low[*later / 4] = u32::MAX;
        }
        let low = NativeDpModelDna::from_fields(low);
        let high = NativeDpModelDna::from_fields(high);
        assert!(low < high);
        assert!(high > low);
        assert_eq!(low.cmp(&low), Ordering::Equal);
    }
}

#[test]
fn all_input_bits_affect_only_their_verified_model_fields() {
    for byte in 0..24 {
        for bit in 0..8 {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[byte] = 1 << bit;
            let key = NativeDpModelDna::from_dhdna(&dna);
            for (field, &(source, shift, mask)) in LAYOUT.iter().enumerate() {
                let expected = if byte == source && bit >= shift {
                    u32::from(((1_u8 << bit) >> shift) & mask)
                } else {
                    0
                };
                assert_eq!(
                    key.fields()[field],
                    expected,
                    "DNA{byte} bit{bit} field{field}"
                );
            }
            assert_eq!(key.fields()[47], 0);
        }
    }
}
