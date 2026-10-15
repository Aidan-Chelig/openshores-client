use super::*;
use crate::protocol::QtWriter;

#[test]
fn concrete_conditions_replace_retain_and_fail_in_unit_order() {
    let profile = FloatEncoding::TypeWidths;
    let mut state = NativeUnitState::default();
    state.receive(&mut QtReader::new(&[0]), profile).unwrap();
    assert!(state.conditions.is_none());
    let mut bytes = vec![2, 0, 2, 6, 8]; // unknown factory, then Fire
    bytes.extend(17_i64.to_be_bytes());
    bytes.push(0xde);
    let mut input = QtReader::new(&bytes);
    state.receive(&mut input, profile).unwrap();
    assert_eq!(input.read_u8().unwrap(), 0xde);
    assert_eq!(state.conditions.as_ref().unwrap().0.len(), 1);
    assert_eq!(state.conditions.as_ref().unwrap().0[0].time_08, 17);
    state.receive(&mut QtReader::new(&[0]), profile).unwrap();
    assert_eq!(state.conditions.as_ref().unwrap().0.len(), 1);
    state.fields.vector_330 = Some([9.; 3]);
    // Replacement clears old entries, appends Fire, then stops at Pregnant.
    let mut bytes = vec![2, 0, 2, 8];
    bytes.extend(23_i64.to_be_bytes());
    bytes.extend([9, 0xde]);
    let mut input = QtReader::new(&bytes);
    assert_eq!(
        state.receive(&mut input, profile).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(state.conditions.as_ref().unwrap().0[0].time_08, 23);
    assert_eq!(state.fields.vector_330, Some([9.; 3]));
    assert_eq!(input.read_u8().unwrap(), 0xde);
    // Even a truncated count clears before failing.
    assert!(state.receive(&mut QtReader::new(&[2, 0]), profile).is_err());
    assert!(state.conditions.as_ref().unwrap().0.is_empty());
}

#[test]
fn every_flag_combination_preserves_order_profile_and_suffix_boundary() {
    for flags in 0..=255 {
        for encoding in [
            FloatEncoding::TypeWidths,
            FloatEncoding::SinglePrecision,
            FloatEncoding::DoublePrecision,
        ] {
            let mut bytes = vec![flags];
            if flags & 1 != 0 {
                let mut writer = QtWriter::default();
                writer.write_string(Some("native 🦀")).unwrap();
                bytes.extend(writer.into_bytes());
                bytes.extend(i32::MIN.to_be_bytes());
                bytes.push(0xfe);
            }
            if flags & 2 != 0 {
                bytes.push(0x42);
            }
            if flags & 4 != 0 {
                for value in [1.25_f32, -2.5, 3.75] {
                    if encoding == FloatEncoding::DoublePrecision {
                        bytes.extend(f64::from(value).to_be_bytes());
                    } else {
                        bytes.extend(value.to_be_bytes());
                    }
                }
            }
            let consumed = bytes.len();
            bytes.push(0xde);
            let mut fields = NativeUnitFields {
                vector_330: Some([9.; 3]),
                ..Default::default()
            };
            let mut calls = 0;
            let mut input = QtReader::new(&bytes);
            let result = fields
                .receive(&mut input, encoding, |input| {
                    calls += 1;
                    assert_eq!(input.read_u8()?, 0x42);
                    Ok(())
                })
                .unwrap();
            assert_eq!(
                result,
                NativeUnitReceive {
                    flags,
                    consumed_bytes: consumed
                }
            );
            assert_eq!(calls, usize::from(flags & 2 != 0));
            assert_eq!(
                fields.vector_330,
                Some(if flags & 4 != 0 {
                    [1.25, -2.5, 3.75]
                } else {
                    [0.; 3]
                })
            );
            assert_eq!(fields.field_3a8, (flags & 1 != 0).then_some(i32::MIN));
            assert_eq!(fields.field_39c, (flags & 1 != 0).then_some(0xfe));
            assert_eq!(input.read_u8().unwrap(), 0xde);
            fields
                .receive(&mut QtReader::new(&[0]), encoding, |_| {
                    panic!("omitted conditions")
                })
                .unwrap();
            assert_eq!(fields.vector_330, Some([0.; 3]));
            assert_eq!(fields.field_39c, (flags & 1 != 0).then_some(0xfe));
        }
    }
}

#[test]
fn partial_identity_survives_but_incomplete_vector_does_not_replace_old_value() {
    let mut bytes = vec![5, 0xff, 0xff, 0xff, 0xff]; // null QString
    bytes.extend((-17_i32).to_be_bytes());
    bytes.push(0x81);
    for value in [1_f32, 2., 3.] {
        bytes.extend(value.to_be_bytes());
    }
    for end in 10..bytes.len() {
        let mut fields = NativeUnitFields {
            vector_330: Some([9.; 3]),
            ..Default::default()
        };
        assert!(
            fields
                .receive(
                    &mut QtReader::new(&bytes[..end]),
                    FloatEncoding::TypeWidths,
                    |_| panic!()
                )
                .is_err()
        );
        assert_eq!(fields.field_3b0, Some(None));
        assert_eq!(fields.field_3a8, Some(-17));
        assert_eq!(fields.field_39c, Some(0x81));
        assert_eq!(fields.vector_330, Some([9.; 3]));
    }
}

#[test]
fn condition_error_stops_before_vector_even_when_vector_would_be_reset() {
    let mut fields = NativeUnitFields {
        vector_330: Some([9.; 3]),
        ..Default::default()
    };
    let mut input = QtReader::new(&[2, 0xde]);
    let error = fields
        .receive(&mut input, FloatEncoding::TypeWidths, |_| {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "nested body unresolved",
            ))
        })
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    assert_eq!(fields.vector_330, Some([9.; 3]));
    assert_eq!(input.read_u8().unwrap(), 0xde);
}

#[test]
fn double_precision_narrows_before_storage_and_preserves_nonfinite_wire_values() {
    for values in [
        [1.0000000001_f64, -0.0, f64::MAX],
        [f64::NAN, f64::INFINITY, f64::NEG_INFINITY],
    ] {
        let mut bytes = vec![4];
        for value in values {
            bytes.extend(value.to_be_bytes());
        }
        let mut fields = NativeUnitFields::default();
        fields
            .receive(
                &mut QtReader::new(&bytes),
                FloatEncoding::DoublePrecision,
                |_| panic!(),
            )
            .unwrap();
        for (actual, wire) in fields.vector_330.unwrap().into_iter().zip(values) {
            let expected = f64::from(wire as f32);
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else {
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
        }
    }
}

#[test]
fn truncated_scalars_keep_only_the_completed_identity_assignments() {
    let mut bytes = vec![1, 0xff, 0xff, 0xff, 0xff];
    bytes.extend(123_i32.to_be_bytes());
    bytes.push(0x81);
    for end in 0..bytes.len() {
        let mut fields = NativeUnitFields {
            field_3b0: Some(Some("old".into())),
            field_3a8: Some(-7),
            field_39c: Some(2),
            vector_330: Some([9.; 3]),
        };
        assert!(
            fields
                .receive(
                    &mut QtReader::new(&bytes[..end]),
                    FloatEncoding::TypeWidths,
                    |_| panic!()
                )
                .is_err()
        );
        assert_eq!(
            fields.field_3b0,
            if end >= 5 {
                Some(None)
            } else {
                Some(Some("old".into()))
            }
        );
        assert_eq!(fields.field_3a8, Some(if end >= 9 { 123 } else { -7 }));
        assert_eq!(fields.field_39c, Some(2));
        assert_eq!(fields.vector_330, Some([9.; 3]));
    }
}
