use super::*;

#[test]
fn dimension_fields_match_independently_rounded_instruction_fixtures() {
    // Python struct rounds each SSE single operation; the 1.3 multiplier is
    // double. These are not observations from executing the native client.
    let cases = [
        (
            [0; 24],
            [
                0x3e4fd1a7, 0x3ccfd1a7, 0x3e1fdc59, 0x3c9fdc59, 0x3d5525cc, 0x3d5525cc, 0x3d5525cc,
                0x3e4fd1a7, 0x3ccfd1a7, 0, 0x3e1fdc59, 0x3d5525cc, 0x3d5525cc, 0x3e1fdc59,
                0x3c9fdc59,
            ],
        ),
        (
            [255; 24],
            [
                0x40eebb2a, 0x402f5173, 0x40b7a3aa, 0x4006dc31, 0x401629d4, 0x40343232, 0x400725a6,
                0x40eebb2a, 0x402f5173, 0x4189bac0, 0x40b7a3aa, 0x409629d4, 0x40613ebe, 0x40b7a3aa,
                0x4006dc31,
            ],
        ),
        (
            std::array::from_fn(|i| i as u8 * 11),
            [
                0x3f679dc8, 0x3e10c29d, 0x3f8e887b, 0x3e708650, 0x3f8c1557, 0x3f752557, 0x3fa36e3a,
                0x402221a5, 0x3f4087f4, 0x41632984, 0x40177103, 0x3faf1aac, 0x3fe978e5, 0x3f859ff3,
                0x3e8df9f2,
            ],
        ),
    ];
    for (raw, bits) in cases {
        assert_eq!(
            NativeDpDimensions::from_dna(&NativeDhDna(raw))
                .fields_4080_through_40b8()
                .map(f32::to_bits),
            bits
        );
    }
}

#[test]
fn all_size_and_shape_flags_produce_finite_native_fields() {
    for size in 0..32 {
        for flags in 0..=255 {
            let mut dna = NativeDhDna([0xff; 24]);
            dna.0[13] = size << 3;
            dna.0[5] = flags;
            for legs in 0..4 {
                dna.0[12] = legs << 2;
                let fields = NativeDpDimensions::from_dna(&dna);
                assert!(
                    fields
                        .fields
                        .iter()
                        .all(|value| value.is_finite() && *value >= 0.0)
                );
                // Original pre-table widths remain independent of neck count.
                assert!(fields.fields[10] > 0.0);
                assert!(fields.fields[11] > 0.0);
                assert!(fields.fields[12] > 0.0);
            }
        }
    }
}
