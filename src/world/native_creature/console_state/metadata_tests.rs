use super::*;

#[test]
fn current_type_and_station_getters_follow_bound_virtuals_not_arbitrary_wire_shorts() {
    for kind in 0..=255 {
        for station in [0, 0x7fff, 0x8000, 0xffff] {
            let Ok(mut value) = NativeConsoleState::from_native_constructor(kind, station) else {
                continue;
            };
            assert_eq!(value.console_type(), u32::from(kind));
            let expected = if [1, 2, 4, 10, 12, 13, 15, 21, 33, 34, 35].contains(&kind) {
                station
            } else {
                0
            };
            assert_eq!(value.station_number().unwrap(), expected);
            match &mut value.body {
                NativeConsoleBody::Helm(v) => {
                    v.field_26 = 0xffff;
                    assert_eq!(value.station_number().unwrap(), 0);
                }
                NativeConsoleBody::Weapon(v) => {
                    v.field_a2 = None;
                    assert_eq!(
                        value.station_number().unwrap_err().kind(),
                        io::ErrorKind::Unsupported
                    );
                }
                NativeConsoleBody::Transporter(v) => {
                    v.field_ae = None;
                    assert_eq!(
                        value.station_number().unwrap_err().kind(),
                        io::ErrorKind::Unsupported
                    );
                }
                _ => (),
            }
        }
    }
}

#[test]
fn game_type_changes_with_raw_received_byte_and_empty_type_with_nonzero_predicate() {
    for factory in 0x21..=0x23 {
        let mut value = NativeConsoleState::from_native_constructor(factory, 0xffff).unwrap();
        for raw in 0..=255 {
            let bytes = [3, raw, 0x80, 0xff];
            let result = value
                .receive_wire_qt7(&mut QtReader::new(&bytes), FloatEncoding::TypeWidths)
                .unwrap();
            assert_eq!(result.consumed_bytes, 4);
            assert_eq!(value.console_type(), u32::from(raw));
            assert_eq!(value.station_number().unwrap(), 0x80ff);
            assert_eq!(value.factory_type, factory);
        }
    }
    let mut value = NativeConsoleState::from_native_constructor(0, 0xffff).unwrap();
    assert_eq!(value.console_type(), 0);
    assert_eq!(value.station_number().unwrap(), 0);
    value.body = NativeConsoleBody::Empty {
        field_20_nonzero: true,
    };
    assert_eq!(value.console_type(), 0x3d);
    let result = value
        .receive_wire_qt7(&mut QtReader::new(&[0xde]), FloatEncoding::TypeWidths)
        .unwrap();
    assert_eq!(result.consumed_bytes, 0);
    assert_eq!(value.console_type(), 0x3d);
    assert_eq!(value.station_number().unwrap(), 0);
}

#[test]
fn zero_scalar_constructor_subsets_equal_their_full_zero_wire_updates() {
    assert_eq!(
        NativeStationEngineer::from_native_constructor(0),
        NativeStationEngineer::read(&mut QtReader::new(&[0; 9])).unwrap()
    );
    assert_eq!(
        NativeStationHelm::from_native_constructor(),
        NativeStationHelm::read(&mut QtReader::new(&[0; 7])).unwrap()
    );
    assert_eq!(
        NativeStationShield::from_native_constructor(),
        NativeStationShield::read(&mut QtReader::new(&[0; 32]), FloatEncoding::TypeWidths).unwrap()
    );
}
