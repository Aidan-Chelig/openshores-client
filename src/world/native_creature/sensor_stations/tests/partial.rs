use super::fixtures::{float, vector};
use super::*;

#[test]
fn unknown_minimal_list_stops_are_reported_by_each_enclosing_station() {
    for profile in PROFILES {
        let mut original_list = Vec::new();
        list(&mut original_list, profile);
        let mut first = vec![31];
        first.extend(31_u32.to_be_bytes());
        vector(&mut first, [1., 2., 3.], profile);
        for unknown in [0, 44, 255] {
            for station in 0..3 {
                let mut bytes = match station {
                    0 => sensor(0, 2, profile),
                    1 => weapon(0, 18, profile),
                    _ => transporter(0x30, 0, 32, profile),
                };
                bytes.truncate(bytes.len() - original_list.len());
                bytes.extend([0, 3]);
                bytes.extend(&first);
                bytes.extend([unknown, 0xde, 0xad]);
                let mut input = QtReader::new(&bytes);
                let (result, received) = match station {
                    0 => {
                        let mut value = NativeStationSensor::default();
                        let result = value.receive_wire(&mut input, profile).unwrap();
                        (result, value.list_58)
                    }
                    1 => {
                        let mut value = NativeStationWeapon::default();
                        let result = value.receive_wire(&mut input, profile).unwrap();
                        (result, value.list_78)
                    }
                    _ => {
                        let mut value = NativeStationTransporter::default();
                        let result = value.receive_wire(&mut input, profile).unwrap();
                        (result, value.list_78)
                    }
                };
                assert_eq!(received.0.len(), 1);
                assert_eq!(received.0[0].kind, 31);
                assert_eq!(result.minimal_list.declared_count, 3);
                assert_eq!(result.minimal_list.unknown_type, Some(unknown));
                assert_eq!(result.minimal_list.unread_entries, 1);
                assert_eq!(result.minimal_list.consumed_bytes, 3 + first.len());
                assert_eq!(result.consumed_bytes, bytes.len() - 2);
                assert_eq!(input.remaining_bytes(), &[0xde, 0xad]);
            }
        }
    }
}

#[test]
fn sensor_errors_preserve_earlier_ordered_writes_and_clear_list_before_count() {
    for profile in PROFILES {
        let mut old = NativeStationSensor::default();
        let _result = old
            .receive_wire(&mut QtReader::new(&sensor(0, 18, profile)), profile)
            .unwrap();
        old.vector_20 = Some([91., 92., 93.]);
        old.field_38 = Some(94.);
        old.field_3c = Some(95.);
        old.id_1c = Some(96);
        let bytes = sensor(0xffff, 2, profile);
        let float_width = if profile == FloatEncoding::DoublePrecision {
            8
        } else {
            4
        };
        let mut full_body = Vec::new();
        full(&mut full_body, 2, profile);
        let full_end = 2 + full_body.len();
        let vector_end = full_end + 3 * float_width;
        let list_start = vector_end + 2 * float_width + 4;
        let first_min_end = list_start + 2 + 1 + 4 + 3 * float_width;
        for end in 0..bytes.len() {
            let mut value = old.clone();
            assert!(
                value
                    .receive_wire(&mut QtReader::new(&bytes[..end]), profile)
                    .is_err()
            );
            assert_eq!(
                value.flags,
                if end >= 2 {
                    Some(NativeStationSensorFlags {
                        fields_14: [1; 4],
                        fields_18: [1; 2],
                        field_1a: 1,
                        fields_40: [1; 4],
                    })
                } else {
                    old.flags
                }
            );
            if end < 3 {
                assert_eq!(value.blip_50, old.blip_50);
            } else if end < full_end {
                assert_eq!(value.blip_50.kind, Some(2));
                assert!(value.blip_50.value.is_none());
            } else {
                assert_selection(&value.blip_50, 2, profile);
            }
            assert_eq!(
                value.vector_20,
                if end >= vector_end {
                    Some([1., -0., -10.5])
                } else {
                    old.vector_20
                }
            );
            assert_eq!(
                value.field_38,
                if end >= vector_end + float_width {
                    Some(11.25)
                } else {
                    old.field_38
                }
            );
            assert_eq!(
                value.field_3c,
                if end >= vector_end + 2 * float_width {
                    Some(-12.75)
                } else {
                    old.field_3c
                }
            );
            assert_eq!(
                value.id_1c,
                if end >= list_start {
                    Some(0xfedcba98)
                } else {
                    old.id_1c
                }
            );
            if end < list_start {
                assert_eq!(value.list_58, old.list_58);
            } else if end < first_min_end {
                assert!(value.list_58.0.is_empty());
            } else {
                assert_eq!(value.list_58.0[0].kind, 31);
                assert!(value.list_58.0.len() < 4);
            }
        }
    }
}

#[test]
fn transporter_destroys_old_selection_before_reading_optional_type_and_updates_presence_late() {
    for profile in PROFILES {
        let mut old = NativeStationTransporter::default();
        let _result = old
            .receive_wire(
                &mut QtReader::new(&transporter(0x30, 0xffff, 18, profile)),
                profile,
            )
            .unwrap();
        old.field_18 = Some(99);
        old.field_1c = Some(100);
        old.vector_50 = Some([91., 92., 93.]);
        old.field_4d = Some(false);
        let bytes = transporter(0x33, 0, 2, profile);
        let mut full_body = Vec::new();
        full(&mut full_body, 2, profile);
        let full_end = 2 + full_body.len();
        let float_width = if profile == FloatEncoding::DoublePrecision {
            8
        } else {
            4
        };
        for end in 0..bytes.len() {
            let mut value = old.clone();
            assert!(
                value
                    .receive_wire(&mut QtReader::new(&bytes[..end]), profile)
                    .is_err()
            );
            assert_eq!(
                value.field_18,
                if end >= 1 { Some(3) } else { old.field_18 }
            );
            assert_eq!(
                value.field_1c,
                if end >= 2 { Some(0xfe) } else { old.field_1c }
            );
            if end < 2 {
                assert_eq!(value.blip_70, old.blip_70);
            } else if end == 2 {
                assert_eq!(value.blip_70, NativeStationBlipSelection::default());
            } else if end < full_end {
                assert_eq!(value.blip_70.kind, Some(2));
                assert!(value.blip_70.value.is_none());
            } else {
                assert_selection(&value.blip_70, 2, profile);
            }
            let complete = end >= full_end + 3 * float_width;
            assert_eq!(
                value.vector_50,
                if complete {
                    Some([21.5, -22.75, 1.])
                } else {
                    old.vector_50
                }
            );
            assert_eq!(
                value.field_4d,
                if complete { Some(true) } else { old.field_4d }
            );
        }
        // A cleared bit20 sets false before reading the following mandatory vector.
        let mut value = old.clone();
        value.field_4d = Some(true);
        assert!(
            value
                .receive_wire(&mut QtReader::new(&[0, 0xfe]), profile)
                .is_err()
        );
        assert_eq!(value.vector_50, old.vector_50);
        assert_eq!(value.field_4d, Some(false));
        assert_eq!(value.blip_70, NativeStationBlipSelection::default());
    }
}

#[test]
fn raw_nonfinite_float_vectors_and_scalar_narrowing_survive_the_wire_stage() {
    for profile in PROFILES {
        let mut bytes = vec![0, 0, 0]; // flags and null full factory.
        vector(
            &mut bytes,
            [f64::NAN, f64::INFINITY, f64::NEG_INFINITY],
            profile,
        );
        float(&mut bytes, f64::MAX, profile);
        float(&mut bytes, -0., profile);
        bytes.extend([0; 6]); // ID then empty count.
        let mut value = NativeStationSensor::default();
        let result = value
            .receive_wire(&mut QtReader::new(&bytes), profile)
            .unwrap();
        assert_eq!(result.consumed_bytes, bytes.len());
        assert_eq!(result.null_full_factory, Some(0));
        let vector = value.vector_20.unwrap();
        assert!(vector[0].is_nan());
        assert_eq!(vector[1], f64::INFINITY);
        assert_eq!(vector[2], f64::NEG_INFINITY);
        assert_eq!(value.field_38, Some(f32::INFINITY));
        assert_eq!(value.field_3c.unwrap().to_bits(), (-0_f32).to_bits());
        assert!(value.list_58.0.is_empty());
    }
}
