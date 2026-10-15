use super::*;
mod fixtures;
mod partial;
mod weapon_partial;
use fixtures::*;

fn assert_result(result: NativeStationBlipWireReceive, kind: u8, length: usize) {
    assert_eq!(result.consumed_bytes, length);
    assert_eq!(
        result.null_full_factory,
        (!(1..=43).contains(&kind)).then_some(kind)
    );
    assert_eq!(result.minimal_list.declared_count, 4);
    assert_eq!(result.minimal_list.unknown_type, None);
    assert_eq!(result.minimal_list.unread_entries, 0);
}

#[test]
fn all_three_stations_compose_real_full_and_minimal_bodies_in_every_profile() {
    for profile in PROFILES {
        for kind in [2, 18, 31, 32, 0, 44, 255] {
            let mut bytes = sensor(0xffff, kind, profile);
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let mut value = NativeStationSensor::default();
            assert_result(
                value.receive_wire(&mut input, profile).unwrap(),
                kind,
                length,
            );
            assert_eq!(value.vector_20, Some([1., -0., -10.5]));
            assert_eq!(value.vector_20.unwrap()[1].to_bits(), (-0_f64).to_bits());
            assert_eq!(
                (value.field_38, value.field_3c, value.id_1c),
                (Some(11.25), Some(-12.75), Some(0xfedcba98))
            );
            assert_selection(&value.blip_50, kind, profile);
            assert_list(&value.list_58);
            assert_eq!(input.read_u8().unwrap(), 0xde);

            let mut bytes = weapon(0xffff, kind, profile);
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let mut value = NativeStationWeapon::default();
            assert_result(
                value.receive_wire(&mut input, profile).unwrap(),
                kind,
                length,
            );
            assert_eq!(value.field_41, Some(0xfe));
            assert_eq!(value.vector_20, Some([1., -0., -10.5]));
            assert_eq!(
                (value.field_38, value.field_3c, value.field_4c),
                (Some(11.25), Some(0xfd), Some(-12.75))
            );
            assert_eq!(
                (value.field_50, value.field_a2, value.field_44, value.id_1c),
                (Some(0x80ff), Some(0xffff), Some(i32::MIN), Some(0xfedcba98))
            );
            assert_selection(&value.blip_70, kind, profile);
            assert_list(&value.list_78);
            assert_eq!(input.read_u8().unwrap(), 0xde);

            let mut bytes = transporter(0xff, 0xffff, kind, profile);
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let mut value = NativeStationTransporter::default();
            assert_result(
                value.receive_wire(&mut input, profile).unwrap(),
                kind,
                length,
            );
            assert_eq!(
                (value.field_18, value.field_1c, value.field_4d),
                (Some(3), Some(0xfe), Some(true))
            );
            assert_eq!(value.vector_50, Some([21.5, -22.75, 1.]));
            assert_eq!(value.vector_20, Some([1., -0., -10.5]));
            assert_eq!(
                (value.field_38, value.field_44, value.field_ae, value.id_14),
                (Some(11.25), Some(-12.75), Some(0x80ff), Some(0xfedcba98))
            );
            assert_selection(&value.blip_70, kind, profile);
            assert_list(&value.list_78);
            assert_eq!(input.read_u8().unwrap(), 0xde);
        }
    }
}

#[test]
fn all_sixteen_flag_bits_use_native_byte_offset_order_and_direction_precedence() {
    let profile = FloatEncoding::TypeWidths;
    for flags in 0..=u16::MAX {
        let mut sensor_value = NativeStationSensor::default();
        let _result = sensor_value
            .receive_wire(&mut QtReader::new(&sensor(flags, 0, profile)), profile)
            .unwrap();
        let state = sensor_value.flags.unwrap();
        assert_eq!(
            state.fields_14,
            [bit(flags, 0), bit(flags, 1), bit(flags, 2), bit(flags, 3)]
        );
        assert_eq!(state.fields_18, [bit(flags, 4), bit(flags, 5)]);
        assert_eq!(state.field_1a, bit(flags, 6));
        assert_eq!(
            state.fields_40,
            [
                bit(flags, 12),
                bit(flags, 15),
                bit(flags, 13),
                bit(flags, 14)
            ]
        );

        let mut weapon_value = NativeStationWeapon::default();
        let _result = weapon_value
            .receive_wire(&mut QtReader::new(&weapon(flags, 0, profile)), profile)
            .unwrap();
        let state = weapon_value.flags.unwrap();
        assert_eq!(
            state.fields_14,
            [bit(flags, 0), bit(flags, 1), bit(flags, 2), bit(flags, 3)]
        );
        assert_eq!(state.field_18, bit(flags, 4));
        assert_eq!(
            state.fields_3d,
            [
                bit(flags, 12),
                bit(flags, 15),
                bit(flags, 13),
                bit(flags, 14)
            ]
        );
        assert_eq!(
            state.field_48,
            if flags & 64 != 0 {
                1
            } else if flags & 128 != 0 {
                255
            } else {
                0
            }
        );

        let mut transporter_value = NativeStationTransporter::default();
        let _result = transporter_value
            .receive_wire(
                &mut QtReader::new(&transporter(0, flags, 0, profile)),
                profile,
            )
            .unwrap();
        let state = transporter_value.flags.unwrap();
        assert_eq!(state.fields_40, [bit(flags, 2), bit(flags, 3)]);
        assert_eq!(
            state.fields_3c,
            [bit(flags, 4), bit(flags, 5), bit(flags, 0), bit(flags, 1)]
        );
        assert_eq!(state.field_42, bit(flags, 6));
        assert_eq!(state.field_4c, bit(flags, 14));
        assert_eq!(
            state.fields_48,
            [
                bit(flags, 11),
                bit(flags, 12),
                bit(flags, 15),
                bit(flags, 13)
            ]
        );
    }
}

#[test]
fn every_transporter_presence_flag_retains_omitted_vector_but_clears_omitted_blip() {
    for profile in PROFILES {
        let mut value = NativeStationTransporter::default();
        for flags in 0..=255 {
            let _result = value
                .receive_wire(
                    &mut QtReader::new(&transporter(0x30, 0, 2, profile)),
                    profile,
                )
                .unwrap();
            value.vector_50 = Some([91., 92., 93.]);
            let bytes = transporter(flags, 0xa55a, 18, profile);
            let result = value
                .receive_wire(&mut QtReader::new(&bytes), profile)
                .unwrap();
            assert_eq!(result.null_full_factory, None);
            assert_eq!(result.consumed_bytes, bytes.len());
            assert_eq!(value.field_18, Some(u32::from(flags & 3)));
            assert_eq!(value.field_1c, Some(0xfe));
            assert_eq!(value.field_4d, Some(flags & 0x20 != 0));
            assert_eq!(
                value.vector_50,
                Some(if flags & 0x20 != 0 {
                    [21.5, -22.75, 1.]
                } else {
                    [91., 92., 93.]
                })
            );
            if flags & 0x10 != 0 {
                assert_selection(&value.blip_70, 18, profile);
            } else {
                assert_eq!(value.blip_70, NativeStationBlipSelection::default());
            }
            assert_list(&value.list_78);
        }
    }
}

#[test]
fn every_truncated_station_prefix_fails_without_inventing_a_skip() {
    for profile in PROFILES {
        for kind in [0, 2, 18, 32] {
            let bytes = sensor(0xffff, kind, profile);
            for end in 0..bytes.len() {
                assert!(
                    NativeStationSensor::default()
                        .receive_wire(&mut QtReader::new(&bytes[..end]), profile)
                        .is_err()
                );
            }
            let bytes = weapon(0xffff, kind, profile);
            for end in 0..bytes.len() {
                assert!(
                    NativeStationWeapon::default()
                        .receive_wire(&mut QtReader::new(&bytes[..end]), profile)
                        .is_err()
                );
            }
            for flags in [0, 0x10, 0x20, 0x30] {
                let bytes = transporter(flags, 0xffff, kind, profile);
                for end in 0..bytes.len() {
                    assert!(
                        NativeStationTransporter::default()
                            .receive_wire(&mut QtReader::new(&bytes[..end]), profile)
                            .is_err()
                    );
                }
            }
        }
    }
}

#[test]
fn all_null_full_factory_bytes_continue_without_body_consumption() {
    for kind in (0..=255).filter(|kind| !(1..=43).contains(kind)) {
        let profile = FloatEncoding::TypeWidths;
        let a = NativeStationSensor::default()
            .receive_wire(&mut QtReader::new(&sensor(0, kind, profile)), profile)
            .unwrap();
        let b = NativeStationWeapon::default()
            .receive_wire(&mut QtReader::new(&weapon(0, kind, profile)), profile)
            .unwrap();
        let c = NativeStationTransporter::default()
            .receive_wire(
                &mut QtReader::new(&transporter(0x10, 0, kind, profile)),
                profile,
            )
            .unwrap();
        for result in [a, b, c] {
            assert_eq!(result.null_full_factory, Some(kind));
            assert_eq!(result.minimal_list.unknown_type, None);
        }
    }
}
