use super::*;
use fixtures::*;

#[path = "fixtures.rs"]
mod fixtures;

#[test]
fn all_full_factory_types_profiles_fields_and_truncated_prefixes() {
    for profile in PROFILES {
        for kind in 1..=43 {
            for (flags, byte) in [(0, 0), (1, 0x80), (2, 0xff), (0xff, 1)] {
                let (mut bytes, expected) = fixture(kind, profile, flags, byte);
                let length = bytes.len();
                bytes.push(0xde);
                let mut input = QtReader::new(&bytes);
                let value = NativeSensorBlipFull::read(kind, &mut input, profile)
                    .unwrap()
                    .unwrap();
                assert_eq!(value.kind, kind);
                assert_eq!(value.id_08, 0xfedcba98);
                for (value, expected) in value.position_10.into_iter().zip([1., -0_f64, -17.25]) {
                    assert_eq!(value.to_bits(), expected.to_bits());
                }
                assert_eq!(value.body, expected, "kind {kind}, profile {profile:?}");
                if let NativeBlipFullBody::Spacecraft { fields, .. } = &value.body {
                    assert_eq!(fields.point_58[1].to_bits(), (-0_f64).to_bits());
                    assert_eq!(
                        fields.point_58[0],
                        if profile == FloatEncoding::SinglePrecision {
                            1.
                        } else {
                            1.0000000001
                        }
                    );
                }
                assert_eq!(input.remaining(), 1);
                assert_eq!(input.read_u8().unwrap(), 0xde);
                for end in 0..length {
                    let mut short = QtReader::new(&bytes[..end]);
                    assert!(
                        NativeSensorBlipFull::read(kind, &mut short, profile).is_err(),
                        "kind {kind}, prefix {end}"
                    );
                    assert_eq!(short.remaining(), end, "fresh body errors must be atomic");
                }
            }
        }
    }
}

#[test]
fn all_raw_byte_values_boolean_conversion_zero_repair_and_bd_flags() {
    for profile in PROFILES {
        for byte in 0..=255 {
            for kind in [
                1, 2, 4, 7, 8, 9, 10, 13, 14, 15, 16, 17, 18, 19, 20, 23, 24, 29, 32, 33, 34, 35,
                36, 37, 39, 43,
            ] {
                let (mut bytes, expected) = fixture(kind, profile, byte, byte);
                bytes.push(0xde);
                let mut input = QtReader::new(&bytes);
                assert_eq!(
                    NativeSensorBlipFull::read(kind, &mut input, profile)
                        .unwrap()
                        .unwrap()
                        .body,
                    expected
                );
                assert_eq!(input.read_u8().unwrap(), 0xde);
            }
        }
    }
}

#[test]
fn both_packed_star_bytes_preserve_the_exact_native_bit_fields() {
    for first in 0..=255 {
        for second in 0..=255 {
            let (bytes, expected) = fixture(22, FloatEncoding::TypeWidths, first, second);
            assert_eq!(
                NativeSensorBlipFull::read(
                    22,
                    &mut QtReader::new(&bytes),
                    FloatEncoding::TypeWidths
                )
                .unwrap()
                .unwrap()
                .body,
                expected
            );
        }
    }
}

#[test]
fn unknown_factories_consume_nothing_and_do_not_guess_a_body() {
    for profile in PROFILES {
        for kind in 0..=255 {
            if (1..=43).contains(&kind) {
                continue;
            }
            let mut input = QtReader::new(&[0xde, 0xad]);
            assert_eq!(
                NativeSensorBlipFull::read(kind, &mut input, profile).unwrap(),
                None
            );
            assert_eq!(input.remaining(), 2);
        }
    }
}

#[test]
fn nullable_empty_and_raw_species_are_distinct_from_names() {
    for species in [None, Some(vec![]), Some(vec![0xff, 0, 0x80])] {
        for name in [None, Some(""), Some("人🚀")] {
            let mut bytes = prefix(FloatEncoding::TypeWidths);
            bytes.extend([0xff, 0xff, 0xff, 0xff, 0xfa]);
            match &species {
                None => bytes.extend(u32::MAX.to_be_bytes()),
                Some(raw) => {
                    bytes.extend((raw.len() as u32).to_be_bytes());
                    bytes.extend(raw);
                }
            }
            string(&mut bytes, name);
            let value = NativeSensorBlipFull::read(
                3,
                &mut QtReader::new(&bytes),
                FloatEncoding::TypeWidths,
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                value.body,
                NativeBlipFullBody::Creature(NativeBlipCreature {
                    empire: NativeBlipEmpireFields {
                        field_28: -1,
                        field_2c: 0xfa
                    },
                    species_30: NativeSpecies(species.clone()),
                    suffix: NativeBlipCreatureSuffix::Name {
                        field_40: name.map(str::to_owned)
                    },
                })
            );
        }
    }
}

#[test]
fn raw_nonfinite_positions_survive_without_rendering_sanitization() {
    for profile in PROFILES {
        let mut bytes = 0_u32.to_be_bytes().to_vec();
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            float(&mut bytes, value, profile);
        }
        let value = NativeSensorBlipFull::read(31, &mut QtReader::new(&bytes), profile)
            .unwrap()
            .unwrap();
        assert!(value.position_10[0].is_nan());
        assert_eq!(value.position_10[1], f64::INFINITY);
        assert_eq!(value.position_10[2], f64::NEG_INFINITY);
    }
}

#[test]
fn mixed_full_bodies_leave_the_next_discriminator_exactly_aligned() {
    for profile in PROFILES {
        let mut bytes = Vec::new();
        for kind in 1..=43 {
            bytes.push(kind);
            bytes.extend(fixture(kind, profile, 3, 0xfd).0);
        }
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        for kind in 1..=43 {
            assert_eq!(input.read_u8().unwrap(), kind);
            let value = NativeSensorBlipFull::read(kind, &mut input, profile)
                .unwrap()
                .unwrap();
            assert_eq!(value.body, fixture(kind, profile, 3, 0xfd).1);
        }
        assert_eq!(input.remaining(), 1);
        assert_eq!(input.read_u8().unwrap(), 0xde);
    }
}

#[test]
fn malformed_strings_and_species_budget_fail_without_advancing_the_parent() {
    for profile in PROFILES {
        for tail in [&[0, 0, 0, 1, 0][..], &[0, 0, 0, 2, 0xd8, 0][..]] {
            let mut bytes = prefix(profile);
            bytes.extend(tail);
            let mut input = QtReader::new(&bytes);
            assert!(NativeSensorBlipFull::read(30, &mut input, profile).is_err());
            assert_eq!(input.remaining(), bytes.len());
        }
        let mut bytes = prefix(profile);
        bytes.extend([0; 5]); // Empire fields.
        bytes.extend(1_048_577_u32.to_be_bytes());
        bytes.resize(bytes.len() + 1_048_577, 0xff);
        bytes.push(0);
        let mut input = QtReader::new(&bytes);
        let error = NativeSensorBlipFull::read(2, &mut input, profile).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(input.remaining(), bytes.len());
    }
}
