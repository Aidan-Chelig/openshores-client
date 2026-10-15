use super::*;

fn float(bytes: &mut Vec<u8>, value: f64, encoding: FloatEncoding) {
    if encoding == FloatEncoding::DoublePrecision {
        bytes.extend(value.to_be_bytes());
    } else {
        bytes.extend((value as f32).to_be_bytes());
    }
}
fn double(bytes: &mut Vec<u8>, value: f64, encoding: FloatEncoding) {
    if encoding == FloatEncoding::SinglePrecision {
        bytes.extend((value as f32).to_be_bytes());
    } else {
        bytes.extend(value.to_be_bytes());
    }
}

#[test]
fn nav_all_flags_and_profiles_keep_three_vectors_two_ids_and_real_route_aligned() {
    let vectors = [
        [1.0000000001, -0., f64::MAX],
        [f64::NAN, f64::INFINITY, f64::NEG_INFINITY],
        [7.25, -8.5, 9.75],
    ];
    for encoding in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        for flags in 0..=255_u8 {
            let mut bytes = vec![flags, 0x80, 0xfe, 0xff];
            for vector in &vectors[..2] {
                for value in vector {
                    float(&mut bytes, *value, encoding);
                }
            }
            bytes.extend(0x80000000_u32.to_be_bytes());
            bytes.extend(u32::MAX.to_be_bytes());
            bytes.extend([0xab, 0xcd]);
            for value in vectors[2] {
                float(&mut bytes, value, encoding);
            }
            bytes.extend(1_u16.to_be_bytes());
            bytes.extend(0xfedcba98_u32.to_be_bytes());
            bytes.extend(17_u32.to_be_bytes());
            let waypoint = [1.0000000001_f64, -0., f64::MAX];
            for value in waypoint {
                double(&mut bytes, value, encoding);
            }
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let value = NativeStationNav::read(&mut input, encoding).unwrap();
            assert_eq!(
                value.field_14,
                u32::from_le_bytes([
                    u8::from(flags & 64 != 0),
                    flags & 1,
                    u8::from(flags & 2 != 0),
                    u8::from(flags & 4 != 0)
                ])
            );
            assert_eq!(value.field_18, u8::from(flags & 8 != 0));
            assert_eq!(value.field_5c, (flags / 16) % 4);
            assert_eq!(
                (value.field_1c, value.field_64, value.field_20),
                (128, 254, 255)
            );
            assert_eq!(
                (value.id_68, value.id_58, value.field_6c, value.field_60),
                (0x80000000, u32::MAX, 0xab, 0xcd)
            );
            for (actual, expected) in [value.vector_28, value.vector_40, value.vector_70]
                .into_iter()
                .flatten()
                .zip(vectors.into_iter().flatten())
            {
                let expected = f64::from(expected as f32);
                if expected.is_nan() {
                    assert!(actual.is_nan());
                } else {
                    assert_eq!(actual.to_bits(), expected.to_bits());
                }
            }
            assert_eq!(value.route_90.0.len(), 1);
            assert_eq!(value.route_90.0[0].ids, [0xfedcba98, 17]);
            for (actual, expected) in value.route_90.0[0].position.into_iter().zip(waypoint) {
                let expected = if encoding == FloatEncoding::SinglePrecision {
                    f64::from(expected as f32)
                } else {
                    expected
                };
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
            assert_eq!(input.read_u8().unwrap(), 0xde);
            if flags == 0xff {
                for end in 0..length {
                    let mut short = QtReader::new(&bytes[..end]);
                    assert!(NativeStationNav::read(&mut short, encoding).is_err());
                    assert_eq!(short.remaining(), end);
                }
            }
        }
    }
}

#[test]
fn route_clears_only_after_count_and_appends_only_completed_entries() {
    let old = NativeWaypoint {
        ids: [9, 9],
        position: [9.; 3],
    };
    let mut bytes = 2_u16.to_be_bytes().to_vec();
    for id in [1_u32, 2] {
        bytes.extend(id.to_be_bytes());
        bytes.extend(u32::MAX.to_be_bytes());
        for value in [1.25_f64, 2.5, 3.75] {
            bytes.extend(value.to_be_bytes());
        }
    }
    for end in 0..bytes.len() {
        let mut route = NativeWaypointRoute(vec![old.clone()]);
        assert!(
            route
                .receive(&mut QtReader::new(&bytes[..end]), FloatEncoding::TypeWidths)
                .is_err()
        );
        if end < 2 {
            assert_eq!(route.0, vec![old.clone()]);
        } else if end < 34 {
            assert!(route.0.is_empty());
        } else {
            assert_eq!(route.0.len(), 1);
            assert_eq!(route.0[0].ids, [1, u32::MAX]);
            assert_eq!(route.0[0].position, [1.25, 2.5, 3.75]);
        }
    }
    let mut route = NativeWaypointRoute(vec![old]);
    route
        .receive(&mut QtReader::new(&[0, 0]), FloatEncoding::TypeWidths)
        .unwrap();
    assert!(route.0.is_empty());
}

#[test]
fn route_count_high_bits_do_not_turn_into_an_empty_or_negative_loop() {
    for count in [32768_u16, 65535] {
        let mut bytes = count.to_be_bytes().to_vec();
        bytes.extend(std::iter::repeat_n(0, usize::from(count) * 32));
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        let mut route = NativeWaypointRoute::default();
        route
            .receive(&mut input, FloatEncoding::TypeWidths)
            .unwrap();
        assert_eq!(route.0.len(), usize::from(count));
        assert_eq!(input.read_u8().unwrap(), 0xde);
    }
}
