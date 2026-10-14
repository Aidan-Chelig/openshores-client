use super::*;

#[test]
fn literal_record_reads_parent_then_rotation_then_position() {
    // Flags: parent + X rotation + X position; not the server's six-float layout.
    let bytes = [
        0x13, 0xf1, 0x23, 0x45, 0x67, 0x3f, 0x00, 0x00, 0x00, // X rotation = 0.5 radians
        0xc0, 0x00, 0x00, 0x00, // X position = -2
        0xaa,
    ];
    let mut input = QtReader::new(&bytes);
    let record = PackedPosition::read(&mut input, FloatEncoding::TypeWidths).unwrap();
    assert_eq!(record.parent, Some(0xf1234567));
    assert_eq!(record.rotation, [0.5, 0.0, 0.0]);
    assert_eq!(record.position, [-2.0, 0.0, 0.0]);
    assert_eq!(input.read_u8().unwrap(), 0xaa);
}

fn fixture(flags: u8, encoding: FloatEncoding) -> Vec<u8> {
    let mut bytes = vec![flags];
    if flags & 1 != 0 {
        bytes.extend(0xf1234567_u32.to_be_bytes());
    }
    for (bit, value, double) in [
        (0x10, 0.125_f64, false),
        (0x20, -0.25, false),
        (0x40, 1.5, false),
        (2, 12345678.125, flags & 0x80 != 0),
        (4, -42.25, flags & 0x80 != 0),
        (8, 3.5, flags & 0x80 != 0),
    ] {
        if flags & bit != 0 {
            if encoding == FloatEncoding::DoublePrecision
                || (encoding == FloatEncoding::TypeWidths && double)
            {
                bytes.extend(value.to_be_bytes());
            } else {
                bytes.extend((value as f32).to_be_bytes());
            }
        }
    }
    bytes
}

#[test]
fn all_flags_and_stream_encodings_preserve_order_defaults_and_following_fields() {
    for encoding in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        for flags in 0..=u8::MAX {
            let mut bytes = fixture(flags, encoding);
            let size = bytes.len();
            bytes.push(0xaa);
            let mut input = QtReader::new(&bytes);
            let record = PackedPosition::read(&mut input, encoding).unwrap();
            assert_eq!(record.flags, flags);
            assert_eq!(record.parent, (flags & 1 != 0).then_some(0xf1234567));
            for (index, value) in [0.125, -0.25, 1.5].into_iter().enumerate() {
                assert_eq!(
                    record.rotation[index],
                    if flags & (0x10 << index) != 0 {
                        value
                    } else {
                        0.0
                    }
                );
            }
            for (index, value) in [12345678.125, -42.25, 3.5].into_iter().enumerate() {
                let expected = if flags & (2 << index) == 0 {
                    0.0
                } else if flags & 0x80 != 0 && encoding != FloatEncoding::SinglePrecision {
                    value
                } else {
                    f64::from(value as f32)
                };
                assert_eq!(record.position[index], expected);
            }
            assert_eq!(input.remaining(), bytes.len() - size);
            assert_eq!(input.read_u8().unwrap(), 0xaa);
        }
    }
}

#[test]
fn every_truncated_record_is_rejected_without_consumption() {
    for encoding in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        for flags in 0..=u8::MAX {
            let bytes = fixture(flags, encoding);
            for length in 0..bytes.len() {
                let mut input = QtReader::new(&bytes[..length]);
                assert_eq!(
                    PackedPosition::read(&mut input, encoding)
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::UnexpectedEof
                );
                assert_eq!(input.remaining(), length);
            }
        }
    }
}

#[test]
fn nonfinite_and_float_narrowing_overflow_are_rejected_atomically() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
        for flag in [2, 0x10] {
            let mut bytes = vec![flag];
            bytes.extend(value.to_be_bytes());
            let mut input = QtReader::new(&bytes);
            assert_eq!(
                PackedPosition::read(&mut input, FloatEncoding::DoublePrecision)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(input.remaining(), bytes.len());
        }
    }
}

#[test]
fn explicit_null_parent_and_negative_zero_are_preserved() {
    let mut bytes = vec![0x13];
    bytes.extend(0_u32.to_be_bytes());
    bytes.extend((-0.0_f32).to_be_bytes());
    bytes.extend((-0.0_f32).to_be_bytes());
    let record =
        PackedPosition::read(&mut QtReader::new(&bytes), FloatEncoding::TypeWidths).unwrap();
    assert_eq!(record.parent, Some(0));
    assert!(record.rotation[0].is_sign_negative());
    assert!(record.position[0].is_sign_negative());
}
