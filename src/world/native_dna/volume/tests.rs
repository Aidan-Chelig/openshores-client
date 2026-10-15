use super::*;
use crate::protocol::QtReader;

fn decoded(kind: u8, raw: u16) -> NativeChromosome {
    let mut wire = vec![0]; // No gene chain.
    match kind {
        8 => {
            wire.push(0);
            for _ in 0..8 {
                wire.extend(raw.to_be_bytes());
            }
        }
        17 => {
            wire.extend([0, 0]); // Widened byte14 and byte18.
            wire.extend(raw.to_be_bytes());
            wire.extend(raw.to_be_bytes());
        }
        27 => {
            wire.extend(raw.to_be_bytes());
            wire.extend([0, 0]); // Field0c and field14.
        }
        _ => unreachable!(),
    }
    let mut reader = QtReader::new(&wire);
    let chromosome = NativeChromosome::read_qt7(kind, &mut reader)
        .unwrap()
        .unwrap();
    assert_eq!(reader.remaining(), 0);
    chromosome
}

#[test]
fn native_volume_rounding_and_unsigned_short_fixtures() {
    // Independently evaluated SSE instruction order, with an explicit f32
    // round after each scalar operation (Python struct, not Rust helpers).
    for (raw, expected) in [
        (0, [0, 0, 0]),
        (1, [0x329e_d584, 0x313e_99d1, 0x325e_5e1f]),
        (1000, [0x4193_ecfd, 0x4031_82c9, 0x414f_1895]),
        (65535, [0x4a9e_d3a8, 0x493e_9796, 0x4a5e_5b85]),
    ] {
        for (kind, bits) in [8, 17, 27].into_iter().zip(expected) {
            assert_eq!(decoded(kind, raw).native_volume().unwrap().to_bits(), bits);
        }
    }
}

#[test]
fn mammary_field0c_suppresses_only_exact_one() {
    let mut chromosome = decoded(27, 1000);
    for flag in [0, 1, 2, 255] {
        let NativeChromosomeBody::Mammary { field_0c, .. } = &mut chromosome.body else {
            unreachable!()
        };
        *field_0c = flag;
        assert_eq!(
            chromosome.native_volume().unwrap().to_bits(),
            if flag == 1 { 0 } else { 0x414f_1895 }
        );
    }
}

#[test]
fn chromatid_accumulates_in_order_and_propagates_invalid_storage() {
    let large = decoded(8, 65535);
    let small = decoded(8, 1000);
    let a = large.native_volume().unwrap();
    let b = small.native_volume().unwrap();
    assert_eq!(
        native_chromatid_volume(&[large, small.clone(), small])
            .unwrap()
            .to_bits(),
        ((a + b) + b).to_bits()
    );
    assert_eq!(native_chromatid_volume(&[]).unwrap().to_bits(), 0);
    for kind in [9, 10, 11] {
        let chromosome = NativeChromosome {
            kind,
            body: NativeChromosomeBody::Empty,
        };
        assert_eq!(
            native_chromatid_volume(&[chromosome]).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}

#[test]
fn face_volumes_preserve_mixed_precision_and_unequal_branches() {
    for (x, y, z, mouth, nose) in [
        (1000_u16, 1000_u16, 1000_u16, 0x41dd_e37b, 0x41dd_e37b),
        (123, 456, 789, 0x405c_5613, 0x3f35_3a1c),
        (65535, 1, 30000, 0x4232_b2af, 0x4905_2192),
        (0, 1000, 2000, 0x4213_ecfd, 0),
        (1, 65535, 65535, 0x4a1e_d4e6, 0x42ee_3e6a),
    ] {
        for (kind, expected) in [(19, nose), (20, mouth)] {
            let mut wire = vec![0];
            if kind == 19 {
                for short in [0, x, y, z] {
                    wire.extend(short.to_be_bytes());
                }
            } else {
                for short in [x, 0, y, 0] {
                    wire.extend(short.to_be_bytes());
                }
                wire.push(0);
                for short in [0, 0, z] {
                    wire.extend(short.to_be_bytes());
                }
            }
            let mut input = QtReader::new(&wire);
            let chromosome = NativeChromosome::read_qt7(kind, &mut input)
                .unwrap()
                .unwrap();
            assert_eq!(input.remaining(), 0);
            assert_eq!(
                chromosome.native_volume().unwrap().to_bits(),
                expected,
                "kind={kind} ({x},{y},{z})"
            );
            if kind == 19 && y != z {
                let mut swapped = chromosome.clone();
                let NativeChromosomeBody::WithGenes { fields, .. } = &mut swapped.body else {
                    unreachable!()
                };
                for field in fields {
                    if field.offset == 0x14 {
                        field.value = NativeChromosomeValue::Short(z);
                    }
                    if field.offset == 0x16 {
                        field.value = NativeChromosomeValue::Short(y);
                    }
                }
                assert_eq!(swapped.native_volume().unwrap().to_bits(), expected);
            }
        }
    }
}

#[test]
fn missing_malformed_volume_storage_is_not_zero() {
    let mut chromosome = decoded(8, 1000);
    let NativeChromosomeBody::WithGenes { fields, .. } = &mut chromosome.body else {
        unreachable!()
    };
    fields.push(fields.last().unwrap().clone());
    assert_eq!(
        chromosome.native_volume().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    chromosome.body = NativeChromosomeBody::Empty;
    assert_eq!(
        chromosome.native_volume().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    chromosome.kind = 28;
    assert_eq!(
        chromosome.native_volume().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[test]
fn all_shared_base_slots_return_positive_zero() {
    for kind in (0..28).filter(|kind| ![8, 9, 10, 11, 17, 19, 20, 27].contains(kind)) {
        // These methods do not read the chromosome body at all, including
        // Wing, Fin, Tentacle and skin/color chromosomes.
        let chromosome = NativeChromosome {
            kind,
            body: NativeChromosomeBody::Empty,
        };
        assert_eq!(chromosome.native_volume().unwrap().to_bits(), 0, "{kind}");
    }
}
