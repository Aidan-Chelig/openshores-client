use super::*;
use crate::protocol::QtWriter;

fn dna(bytes: &mut Vec<u8>, kind: u8) {
    match kind {
        3 => {
            bytes.extend(24_u32.to_be_bytes());
            bytes.extend([0xff; 24]);
        }
        4 => {
            bytes.extend([0xa5, 0, 1, 13, 1, 0, 1, 1]); // AuDNA, Tentacle, one Hair gene
            bytes.push(1); // QColor spec
            for field in [0xffff_u16, 0x8000, 1, 2, 3] {
                bytes.extend(field.to_be_bytes());
            }
            bytes.extend(0xfedc_u16.to_be_bytes());
            bytes.extend(i64::MIN.to_be_bytes());
            bytes.extend([0xfe, 0xab, 0, 0]); // Tentacle fields, second chromatid empty
        }
        _ => (),
    }
}

#[test]
fn every_pregnant_flag_pattern_obeys_dna_precedence_and_trailing_id_order() {
    for flags in 0..=255_u8 {
        for dynamic_type in [3, 4, 0xff] {
            let mut bytes = i64::MIN.to_be_bytes().to_vec();
            bytes.push(flags);
            let kind = if flags & 0x40 != 0 {
                bytes.push(dynamic_type);
                dna(&mut bytes, dynamic_type);
                dynamic_type
            } else if flags & 0x20 != 0 {
                0xff
            } else {
                dna(&mut bytes, 3);
                3
            };
            bytes.push(flags); // covers all 256 values, including zero without normalization
            let text = match flags & 3 {
                0 => None,
                1 => Some(""),
                _ => Some("offspring 🦀"),
            };
            let mut writer = QtWriter::default();
            writer.write_string(text).unwrap();
            bytes.extend(writer.into_bytes());
            if flags & 0x80 != 0 {
                bytes.extend(0xfedcba98_u32.to_be_bytes());
            }
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let value = NativeCondition::read(9, &mut input, FloatEncoding::TypeWidths)
                .unwrap()
                .unwrap();
            assert_eq!(value.time_08, i64::MIN);
            let NativeConditionBody::Pregnant {
                field_10,
                field_18,
                id_1c,
                field_20,
                field_28,
            } = value.body
            else {
                panic!()
            };
            assert_eq!(field_18, flags);
            assert_eq!(field_20, flags & 3);
            assert_eq!(field_28.as_deref(), text);
            assert_eq!(id_1c, if flags & 0x80 != 0 { 0xfedcba98 } else { 0 });
            match (kind, field_10) {
                (3, Some(NativeDna::Dh(value))) => {
                    assert_eq!(value.0[5], 0x7f);
                    assert_eq!(value.0[23], 0);
                }
                (4, Some(NativeDna::Au(value))) => {
                    assert_eq!(value.version, 0xa5);
                    assert_eq!(value.chromatids[0].len(), 1);
                    assert!(value.chromatids[1].is_empty());
                    assert_eq!(value.chromatids[0][0].kind, 13);
                    let super::super::native_dna::NativeChromosomeBody::WithGenes {
                        genes: Some(genes),
                        ..
                    } = &value.chromatids[0][0].body
                    else {
                        panic!()
                    };
                    assert_eq!(genes.0[0].field_18, 0xfedc);
                    assert_eq!(
                        genes.0[0].suffix,
                        super::super::native_dna::NativeGeneSuffix::Hair { field_20: i64::MIN }
                    );
                }
                (0xff, None) => (),
                _ => panic!("wrong DNA branch for flags {flags:02x}"),
            }
            assert_eq!(input.read_u8().unwrap(), 0xde);
            for end in 0..length {
                let mut short = QtReader::new(&bytes[..end]);
                assert!(NativeCondition::read(9, &mut short, FloatEncoding::TypeWidths).is_err());
                assert_eq!(short.remaining(), end);
            }
        }
    }
}
