use super::*;
use crate::world::native_dna::{NativeChromosomeBody, NativeGeneSuffix};

#[test]
fn ear_and_eye_fields_follow_wire_order_not_object_offset_order() {
    use crate::world::native_dna::{NativeChromosomeField as F, NativeChromosomeValue::*};
    for (kind, expected) in [
        (
            17,
            vec![
                F {
                    offset: 0x14,
                    value: Unsigned32(0x80),
                },
                F {
                    offset: 0x18,
                    value: Byte(0x81),
                },
                F {
                    offset: 0x10,
                    value: Short(0x8283),
                },
                F {
                    offset: 0x12,
                    value: Short(0x8485),
                },
            ],
        ),
        (
            18,
            vec![
                F {
                    offset: 0x10,
                    value: Unsigned32(0x8081),
                },
                F {
                    offset: 0x14,
                    value: Short(0x8283),
                },
                F {
                    offset: 0x18,
                    value: Unsigned32(0x84),
                },
                F {
                    offset: 0x1c,
                    value: Short(0x8586),
                },
                F {
                    offset: 0x1e,
                    value: Byte(0x87),
                },
            ],
        ),
    ] {
        let bytes = chromosome(kind, false);
        let value = NativeChromosome::read_qt7(kind, &mut QtReader::new(&bytes[1..]))
            .unwrap()
            .unwrap();
        let NativeChromosomeBody::WithGenes { genes, fields } = value.body else {
            panic!()
        };
        assert_eq!(genes, None);
        assert_eq!(fields, expected);
    }
}

fn gene(kind: u8) -> Vec<u8> {
    let mut bytes = vec![kind];
    if kind > 3 {
        return bytes;
    }
    bytes.push(0xfe); // signed spec -2, not clamped to a known color enum
    for value in [1_u16, 0x8000, 0xffff, 4, 5] {
        bytes.extend(value.to_be_bytes());
    }
    bytes.extend(0x80ab_u16.to_be_bytes());
    match kind {
        0 => bytes.push(0xfe),
        1 => bytes.extend(i64::MIN.to_be_bytes()),
        _ => (),
    }
    bytes
}

fn chromosome(kind: u8, genes: bool) -> Vec<u8> {
    let mut bytes = vec![kind];
    let body_length = match kind {
        0 | 1 => 0,
        2 | 3 | 21 | 23 | 24 => 1,
        4 | 5 | 7 => 22,
        6 => 12,
        8 | 10 => 17,
        9 => 16,
        11 => 22,
        12 => 5,
        13 => 2,
        14 => 4,
        15 | 26 => 4,
        16 | 22 | 25 => 2,
        17 => 6,
        18 | 19 => 8,
        20 => 15,
        27 => 5,
        _ => return bytes,
    };
    if matches!(kind, 8..=14 | 17..=20) {
        bytes.push(if genes { 0xff } else { 0xfe }); // only low bit matters
        if genes {
            bytes.extend(6_u16.to_be_bytes());
            for kind in [4, 0, 1, 2, 255, 3] {
                bytes.extend(gene(kind));
            }
        }
    }
    bytes.extend((0..body_length).map(|index| 0x80 + index as u8));
    bytes
}

#[test]
fn complete_two_chromatid_dna_covers_every_factory_type_version_and_prefix() {
    for genes in [false, true] {
        let mut bytes = vec![0x7f];
        bytes.extend(30_u16.to_be_bytes());
        for kind in 0..28 {
            bytes.extend(chromosome(kind, genes));
        }
        bytes.extend([28, 255]); // null chromosome factories consume no body
        bytes.extend(1_u16.to_be_bytes());
        bytes.extend(chromosome(13, !genes));
        let length = bytes.len();
        bytes.push(0xde);
        for version in 0..=255 {
            bytes[0] = version;
            let mut input = QtReader::new(&bytes);
            let dna = NativeAuDna::read_qt7(&mut input).unwrap();
            assert_eq!(dna.version, version);
            assert_eq!(dna.chromatids[0].len(), 28);
            assert_eq!(dna.chromatids[1].len(), 1);
            for (kind, chromosome) in dna.chromatids[0].iter().enumerate() {
                assert_eq!(usize::from(chromosome.kind), kind);
                if let NativeChromosomeBody::WithGenes { genes: chain, .. } = &chromosome.body {
                    assert_eq!(chain.is_some(), genes);
                    if let Some(chain) = chain {
                        assert_eq!(
                            chain.0.iter().map(|gene| gene.kind).collect::<Vec<_>>(),
                            vec![0, 1, 2, 3]
                        );
                        for gene in &chain.0 {
                            assert_eq!(gene.color_08.spec, -2);
                            assert_eq!(gene.color_08.components, [1, 0x8000, 0xffff, 4, 5]);
                            assert_eq!(gene.field_18, 0x80ab);
                        }
                        assert_eq!(
                            chain.0[0].suffix,
                            NativeGeneSuffix::Feathers { field_1a: 0xfe }
                        );
                        assert_eq!(
                            chain.0[1].suffix,
                            NativeGeneSuffix::Hair { field_20: i64::MIN }
                        );
                    }
                }
            }
            assert_eq!(input.read_u8().unwrap(), 0xde);
        }
        for end in 0..length {
            let mut input = QtReader::new(&bytes[..end]);
            assert!(NativeAuDna::read_qt7(&mut input).is_err(), "prefix {end}");
            assert_eq!(input.remaining(), end);
        }
    }
}

#[test]
fn dynamic_type_is_not_an_embedded_dhdna_prefix_and_unknown_types_do_not_skip() {
    for kind in 0..=255 {
        let mut bytes = vec![kind];
        match kind {
            3 => {
                bytes.extend(24_u32.to_be_bytes());
                bytes.extend([0xff; 24]);
            }
            4 => bytes.extend([0xff, 0, 0, 0, 0]),
            _ => (),
        }
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        let value = NativeDna::read_qt7(&mut input).unwrap();
        match kind {
            3 => {
                let Some(NativeDna::Dh(value)) = value else {
                    panic!()
                };
                assert_eq!(value.0[5], 0x7f);
                assert_eq!(value.0[23], 0);
            }
            4 => {
                let Some(NativeDna::Au(value)) = value else {
                    panic!()
                };
                assert_eq!(value.version, 255);
                assert_eq!(value.chromatids, [vec![], vec![]]);
            }
            _ => assert_eq!(value, None),
        }
        assert_eq!(input.read_u8().unwrap(), 0xde);
        if matches!(kind, 3 | 4) {
            for end in 0..bytes.len() - 1 {
                let mut short = QtReader::new(&bytes[..end]);
                assert!(NativeDna::read_qt7(&mut short).is_err());
                assert_eq!(short.remaining(), end);
            }
        }
    }
}

#[test]
fn aggregate_budget_covers_null_factories_and_nested_genes_without_allocation_from_count() {
    let mut bytes = vec![0, 0, 1, 13, 1]; // one Tentacle, chain present
    bytes.extend(20_000_u16.to_be_bytes());
    bytes.extend(std::iter::repeat_n(255, 20_000));
    bytes.extend([0, 0, 0, 0]);
    let mut input = QtReader::new(&bytes);
    assert_eq!(
        NativeAuDna::read_qt7(&mut input).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(input.remaining(), bytes.len());
}
