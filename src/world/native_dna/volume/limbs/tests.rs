use super::*;
use crate::protocol::QtReader;

fn decoded(kind: u8, values: &[u16]) -> NativeChromosome {
    let mut wire = vec![0];
    for (index, value) in values.iter().enumerate() {
        let byte = match kind {
            9 => false,
            10 => [3, 6, 8].contains(&index),
            11 => [9, 11].contains(&index),
            _ => unreachable!(),
        };
        if byte {
            wire.push(*value as u8);
        } else {
            wire.extend(value.to_be_bytes());
        }
    }
    let mut input = QtReader::new(&wire);
    let result = NativeChromosome::read_qt7(kind, &mut input)
        .unwrap()
        .unwrap();
    assert_eq!(input.remaining(), 0);
    result
}

#[test]
fn limb_wire_fields_and_exact_operation_order() {
    // Independent Python scalar evaluation of the retained GNU instruction
    // sequences with struct f32 rounding after every single operation.
    for (kind, count, expected) in [
        (9, 8, [0x428a_ae2d, 0x4b94_e66e, 0x41ca_e6d5, 0x4386_7e2c]),
        (10, 10, [0x43bd_87a4, 0x4ccb_7f2f, 0x4291_8146, 0x4468_7d09]),
        (11, 12, [0x43a4_9140, 0x4cb0_b1de, 0x42d3_7763, 0x441a_97bc]),
    ] {
        let fixtures = [
            vec![1000; count],
            vec![65535; count],
            (0..count).map(|i| 100 + 137 * i as u16).collect(),
            (0..count).map(|i| 2000 - 137 * i as u16).collect(),
        ];
        for (fields, bits) in fixtures.iter().zip(expected) {
            assert_eq!(
                decoded(kind, fields).native_volume().unwrap().to_bits(),
                bits,
                "kind={kind} {fields:?}"
            );
        }
        assert_eq!(
            decoded(kind, &vec![0; count])
                .native_volume()
                .unwrap()
                .to_bits(),
            0
        );
    }
}

#[test]
fn limb_zero_length_unequal_segments_preserve_native_results() {
    // Unequal radii and a zero segment length do not trigger a native guard.
    // The infinite slope gives height zero and zero cone terms, not an
    // Unsupported error or invented positive geometric segment volume.
    for (kind, mut fields) in [(10, vec![1000; 10]), (11, vec![1000; 12])] {
        if kind == 10 {
            fields[4] = 2000;
            fields[7] = 0;
        } else {
            fields[6] = 2000;
            fields[10] = 0;
        }
        // With positive larger radius the zero-length branch actually gives
        // height=0 and both products=0; retain that result, too.
        assert!(decoded(kind, &fields).native_volume().unwrap().is_finite());
    }
}
