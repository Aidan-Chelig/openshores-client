use super::*;

#[test]
fn foot_selection_uses_full_runtime_field_ankle_flag_and_actual_dna_key() {
    for byte in 0..=255 {
        let mut dna = NativeDhDna([255; 24]);
        dna.0[10] = byte;
        dna.0[16] = 250;
        for field in [-1, 0, 0x41, 0x50, 0x99, 0x9b, 0x141, 0x199, i32::MAX] {
            for mode in [false, true] {
                let request = NativeDpFootResource::new(&dna, field, mode);
                let pa = matches!(field, 0x41 | 0x50 | 0x99 | 0x9b);
                let ankle = !matches!(byte >> 5, 4 | 6);
                assert_eq!(native_dh_has_ankle_bend(&dna), ankle);
                assert_eq!(
                    request.resource(),
                    if pa {
                        format!(":data/PAFootRight{}.3dsN", u8::from(ankle))
                    } else {
                        format!(":data/2FootRight{}.3dsN", byte >> 5)
                    }
                );
                let mut expected = *NativeDpModelDna::from_dhdna(&dna).fields();
                if pa && !mode {
                    expected[47] = 1;
                }
                assert_eq!(request.model_dna().fields(), &expected);
            }
        }
    }
}
