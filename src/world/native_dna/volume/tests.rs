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
fn chromatid_accumulates_in_order_and_propagates_unsupported() {
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
    for kind in [9, 10, 11, 19, 20] {
        let chromosome = NativeChromosome {
            kind,
            body: NativeChromosomeBody::Empty,
        };
        assert_eq!(
            native_chromatid_volume(&[chromosome]).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
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
