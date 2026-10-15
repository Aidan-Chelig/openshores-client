use super::*;

const PROFILES: [FloatEncoding; 3] = [
    FloatEncoding::TypeWidths,
    FloatEncoding::SinglePrecision,
    FloatEncoding::DoublePrecision,
];
const KINDS: [u8; 23] = [
    0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 19, 20, 21, 33, 34, 35,
];

fn float(out: &mut Vec<u8>, profile: FloatEncoding) {
    out.extend(std::iter::repeat_n(
        0,
        if profile == FloatEncoding::DoublePrecision {
            8
        } else {
            4
        },
    ));
}
fn fixture(kind: u8, profile: FloatEncoding) -> Vec<u8> {
    let mut out = vec![];
    match kind {
        0 | 11 => (),
        1 | 12 => out.resize(9, 0),
        3 | 14 => out.resize(7, 0),
        33..=35 => out.push(0),
        9 | 20 => {
            for _ in 0..6 {
                float(&mut out, profile);
            }
            out.resize(
                out.len()
                    + if profile == FloatEncoding::SinglePrecision {
                        4
                    } else {
                        8
                    },
                0,
            );
        }
        7 | 18 => out.resize(5, 0),
        4 | 15 => out.extend([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0, 0,
        ]),
        5 | 16 => {
            out.extend([0; 4]);
            for _ in 0..6 {
                float(&mut out, profile);
            }
            out.extend([0; 10]);
            for _ in 0..3 {
                float(&mut out, profile);
            }
            out.extend([0; 2]);
        }
        8 | 19 => {
            out.extend([0; 3]);
            for _ in 0..5 {
                float(&mut out, profile);
            }
            out.extend([0; 6]);
        }
        2 | 13 => {
            out.extend([0; 4]);
            for _ in 0..4 {
                float(&mut out, profile);
            }
            out.push(0);
            float(&mut out, profile);
            out.extend([0; 14]);
        }
        10 | 21 => {
            out.extend([0; 2]);
            for _ in 0..4 {
                float(&mut out, profile);
            }
            out.extend([0; 2]);
            float(&mut out, profile);
            out.extend([0; 8]);
        }
        _ => panic!("unsupported fixture"),
    }
    out
}

#[test]
fn factory_accepts_only_verified_native_console_classes_and_preserves_raw_station_arguments() {
    for kind in 0..=255 {
        let result = NativeConsoleState::from_native_constructor(kind, 0x80ff);
        if !KINDS.contains(&kind) {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
            continue;
        }
        let value = result.unwrap();
        assert_eq!(value.factory_type, kind);
        match (kind, value.body) {
            (
                0,
                NativeConsoleBody::Empty {
                    field_20_nonzero: false,
                },
            )
            | (11, NativeConsoleBody::Command) => (),
            (1 | 12, NativeConsoleBody::Engineer(v)) => assert_eq!(v.field_2a, 0x80ff),
            (2 | 13, NativeConsoleBody::Weapon(v)) => assert_eq!(v.field_a2, Some(0x80ff)),
            (3 | 14, NativeConsoleBody::Helm(v)) => assert_eq!(v.field_26, 0),
            (4 | 15, NativeConsoleBody::Surgery(v)) => assert_eq!(v.field_28, 0x80ff),
            (5 | 16, NativeConsoleBody::Nav(v)) => assert_eq!(v.field_20, 3),
            (7 | 18, NativeConsoleBody::Power(v)) => assert_eq!(v.group_01[0].field_3c, Some(17)),
            (8 | 19, NativeConsoleBody::Sensor(v)) => {
                assert_eq!(v.flags.unwrap().fields_14, [1, 1, 1, 0])
            }
            (9 | 20, NativeConsoleBody::Shield(v)) => assert_eq!(v.fields_14, [0.; 6]),
            (10 | 21, NativeConsoleBody::Transporter(v)) => assert_eq!(v.field_ae, Some(0x80ff)),
            (33..=35, NativeConsoleBody::Game(v)) => {
                assert_eq!((v.field_14, v.field_16), (kind, 0x80ff))
            }
            _ => panic!("wrong factory binding"),
        }
    }
    for station in 0..=u16::MAX {
        assert_eq!(
            NativeStationEngineer::from_native_constructor(station).field_2a,
            station
        );
        assert_eq!(
            NativeStationSurgery::from_native_constructor(station).field_28,
            station
        );
        assert_eq!(
            NativeStationWeapon::from_native_constructor(station).field_a2,
            Some(station)
        );
        assert_eq!(
            NativeStationTransporter::from_native_constructor(station).field_ae,
            Some(station)
        );
        assert_eq!(
            NativeStationGame::from_native_constructor(0xff, station).field_16,
            station
        );
    }
}

#[test]
fn constructor_receive_subsets_have_verified_nonzero_defaults_not_observation_none() {
    let power = NativeStationPower::from_native_constructor();
    assert_ne!(power, NativeStationPower::default());
    assert_eq!(power.field_68, Some(0));
    assert_eq!(power.field_6c, Some(0));
    assert_eq!(power.fields_c0, [Some(0); 8]);
    for v in power.group_01 {
        assert_eq!(
            v,
            NativePowerGroup01 {
                field_b8: Some(0),
                field_3c: Some(17),
                field_34: Some(0),
                field_74: Some(0)
            }
        );
    }
    for v in power.group_02 {
        assert_eq!(
            v,
            NativePowerGroup02 {
                field_c8: Some(0),
                field_14: Some(17),
                field_94: Some(0)
            }
        );
    }
    let sensor = NativeStationSensor::from_native_constructor();
    assert_eq!(
        sensor.flags,
        Some(NativeStationSensorFlags {
            fields_14: [1, 1, 1, 0],
            fields_18: [1; 2],
            field_1a: 1,
            fields_40: [1; 4]
        })
    );
    assert_eq!(sensor.field_38.unwrap().to_bits(), 0x492c0d47);
    assert_eq!(sensor.field_3c, Some(0.));
    assert_eq!(sensor.id_1c, Some(0));
    assert_eq!(sensor.vector_20, Some([0.; 3]));
    assert!(sensor.blip_50.value.is_none());
    assert!(sensor.list_58.0.is_empty());
    let weapon = NativeStationWeapon::from_native_constructor(13);
    assert_eq!(
        weapon.flags,
        Some(NativeStationWeaponFlags {
            fields_14: [1; 4],
            field_18: 1,
            field_48: 0,
            fields_3d: [0, 1, 0, 1]
        })
    );
    assert_eq!(weapon.field_38.unwrap().to_bits(), 0x492c0d47);
    assert_eq!(
        (
            weapon.field_3c,
            weapon.field_41,
            weapon.field_50,
            weapon.field_a2,
            weapon.field_44,
            weapon.id_1c
        ),
        (Some(0), Some(0), Some(0), Some(13), Some(0), Some(0))
    );
    assert_eq!(weapon.field_4c, Some(0.));
    assert_eq!(weapon.vector_20, Some([0.; 3]));
    assert!(weapon.list_78.0.is_empty());
    let transport = NativeStationTransporter::from_native_constructor(17);
    assert_eq!(
        transport.flags,
        Some(NativeStationTransporterFlags {
            fields_40: [1; 2],
            fields_3c: [1; 4],
            field_42: 1,
            field_4c: 1,
            fields_48: [0, 1, 1, 1]
        })
    );
    assert_eq!(
        (
            transport.field_18,
            transport.field_1c,
            transport.field_ae,
            transport.id_14
        ),
        (Some(0), Some(0), Some(17), Some(0))
    );
    assert_eq!(transport.field_38, Some(1.));
    assert_eq!(transport.field_44, Some(0.));
    assert_eq!(transport.field_4d, Some(false));
    assert_eq!(transport.vector_50, Some([0.; 3]));
    assert_eq!(transport.vector_20, Some([0.; 3]));
    let nav = NativeStationNav::from_native_constructor();
    assert_eq!(
        (
            nav.field_14,
            nav.field_18,
            nav.field_1c,
            nav.field_20,
            nav.field_60,
            nav.field_64
        ),
        (0x01000000, 0, 0, 3, 6, 0)
    );
    assert_eq!(
        (nav.id_58, nav.id_68, nav.field_5c, nav.field_6c),
        (0, 0, 0, 0)
    );
    assert_eq!(
        (nav.vector_28, nav.vector_40, nav.vector_70),
        ([0.; 3], [0.; 3], [0.; 3])
    );
    assert!(nav.route_90.0.is_empty());
    let surgery = NativeStationSurgery::from_native_constructor(19);
    assert_eq!(
        (
            surgery.field_14,
            surgery.field_16,
            surgery.field_18,
            surgery.field_28,
            surgery.field_38
        ),
        (0, 255, 0, 19, 65535)
    );
    assert_eq!(surgery.field_20, None);
    assert!(surgery.points_40.is_empty());
    assert!(surgery.conditions_48.0.is_empty());
}

#[test]
fn every_constructor_bound_reader_composes_in_all_profiles_with_exact_suffix_and_truncation() {
    for profile in PROFILES {
        for kind in KINDS {
            let mut value = NativeConsoleState::from_native_constructor(kind, 0xffff).unwrap();
            let mut bytes = fixture(kind, profile);
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let result = value.receive_wire_qt7(&mut input, profile).unwrap();
            assert_eq!(result.consumed_bytes, length);
            assert_eq!(input.read_u8().unwrap(), 0xde);
            assert_eq!(
                result.sensor_stage.is_some(),
                [2, 8, 10, 13, 19, 21].contains(&kind)
            );
            if let Some(stage) = result.sensor_stage {
                assert_eq!(stage.minimal_list.declared_count, 0);
                assert_eq!(stage.minimal_list.unknown_type, None);
                assert_eq!(
                    stage.null_full_factory,
                    if [10, 21].contains(&kind) {
                        None
                    } else {
                        Some(0)
                    }
                );
            }
            if let NativeConsoleBody::Game(v) = value.body {
                assert_eq!((v.field_14, v.field_16), (0x22, 0));
                assert_eq!(value.factory_type, kind);
            }
            for end in 0..length {
                let mut value = NativeConsoleState::from_native_constructor(kind, 0xffff).unwrap();
                assert!(
                    value
                        .receive_wire_qt7(&mut QtReader::new(&bytes[..end]), profile)
                        .is_err(),
                    "kind {kind}, end {end}, len {length}"
                );
            }
        }
    }
}

#[test]
fn sparse_power_reception_keeps_real_constructor_values_and_later_updates_not_a_reconstructed_zero()
{
    let mut console = NativeConsoleState::from_native_constructor(7, 0).unwrap();
    let _result = console
        .receive_wire_qt7(
            &mut QtReader::new(&[0, 0, 0, 0, 0]),
            FloatEncoding::TypeWidths,
        )
        .unwrap();
    let NativeConsoleBody::Power(ref state) = console.body else {
        panic!()
    };
    assert!(state.group_01.iter().all(|g| g.field_3c == Some(17)));
    assert!(state.group_02.iter().all(|g| g.field_14 == Some(17)));
    let mut bytes = vec![0, 0, 0, 1, 2];
    for _ in 0..8 {
        bytes.extend([0xff, 0xfe]);
        bytes.extend((-9_i32).to_be_bytes());
    }
    let _result = console
        .receive_wire_qt7(&mut QtReader::new(&bytes), FloatEncoding::TypeWidths)
        .unwrap();
    let _result = console
        .receive_wire_qt7(
            &mut QtReader::new(&[0, 0, 0, 2, 0]),
            FloatEncoding::TypeWidths,
        )
        .unwrap();
    let NativeConsoleBody::Power(ref state) = console.body else {
        panic!()
    };
    assert_eq!(state.field_68, Some(2));
    assert!(state.group_01.iter().all(|g| g.field_3c == Some(17)));
    assert!(state.group_02.iter().all(|g| *g
        == NativePowerGroup02 {
            field_c8: Some(255),
            field_14: Some(254),
            field_94: Some(-9)
        }));
}
