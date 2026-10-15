use super::*;

fn fixture(kind: u8, byte: u8) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.extend(2_440_588_u32.to_be_bytes());
    bytes.extend(123_u32.to_be_bytes());
    bytes.push(2);
    if (6..=13).contains(&kind) {
        bytes.extend(0x8001_u16.to_be_bytes());
        bytes.extend(u32::MAX.to_be_bytes());
        string(None, &mut bytes);
        bytes.extend(i32::MIN.to_be_bytes());
        bytes.extend([byte, 255 - byte]);
        bytes.extend(i32::MAX.to_be_bytes());
        string(Some(""), &mut bytes);
        bytes.extend(77_u32.to_be_bytes());
        string(Some("Cargo 🌊"), &mut bytes);
        if kind == 9 {
            bytes.extend(u16::MAX.to_be_bytes());
        }
    } else if kind == 14 || kind == 15 {
        bytes.extend(u32::MAX.to_be_bytes());
        string(None, &mut bytes);
        bytes.push(byte);
        bytes.extend(i32::MIN.to_be_bytes());
        bytes.extend([byte, 255 - byte]);
        bytes.extend(i32::MAX.to_be_bytes());
        string(Some(""), &mut bytes);
        bytes.extend(77_u32.to_be_bytes());
        string(Some("Food 🌊"), &mut bytes);
    } else if kind == 19 {
        string(None, &mut bytes);
        bytes.extend(u32::MAX.to_be_bytes());
        string(Some(""), &mut bytes);
        bytes.extend(77_u32.to_be_bytes());
        string(Some("Station 🌊"), &mut bytes);
    } else {
        assert_eq!(kind, 26);
        bytes.extend(u32::MAX.to_be_bytes());
        bytes.extend(i32::MIN.to_be_bytes());
        string(None, &mut bytes);
        bytes.extend(77_u32.to_be_bytes());
        string(Some("System 🌊"), &mut bytes);
    }
    bytes
}

fn check_cargo(cargo: &NativeJobCargo, byte: u8) {
    assert_eq!(
        cargo,
        &NativeJobCargo {
            field_10: 0x8001,
            id_14: u32::MAX,
            string_18: None,
            field_20: i32::MIN,
            field_24: byte.max(1),
            ignored_wire_byte: 255 - byte,
            field_28: i32::MAX,
            string_30: Some(String::new()),
            id_38: 77,
            string_40: Some("Cargo 🌊".into()),
        }
    );
}

#[test]
fn delivery_and_travel_types_preserve_real_layouts_normalization_and_alignment() {
    for kind in (6..=15).chain([19, 26]) {
        for byte in 0..=255_u8 {
            let mut bytes = fixture(kind, byte);
            let length = bytes.len();
            bytes.push(0xee);
            let mut input = QtReader::new(&bytes);
            let job = NativeJob::read_qt7(kind, &mut input).unwrap();
            assert_eq!(job.kind, kind);
            assert_eq!(job.time_08, 123);
            assert_eq!(input.read_u8().unwrap(), 0xee);
            match &job.body {
                NativeJobBody::DeliverCargo(cargo) => check_cargo(cargo, byte),
                NativeJobBody::DeliverCargoManufacture { cargo, field_48 } => {
                    check_cargo(cargo, byte);
                    assert_eq!(*field_48, 65535);
                }
                NativeJobBody::DeliverFood(food) => assert_eq!(
                    food,
                    &NativeJobFood {
                        id_10: u32::MAX,
                        string_18: None,
                        field_20: u32::from(byte),
                        field_24: i32::MIN,
                        field_28: byte.max(1),
                        ignored_wire_byte: 255 - byte,
                        field_2c: i32::MAX,
                        string_30: Some(String::new()),
                        id_38: 77,
                        string_40: Some("Food 🌊".into()),
                    }
                ),
                NativeJobBody::DestroyEnemyStation {
                    string_10,
                    id_18,
                    string_20,
                    id_28,
                    string_30,
                } => {
                    assert_eq!(string_10, &None);
                    assert_eq!(*id_18, u32::MAX);
                    assert_eq!(string_20.as_deref(), Some(""));
                    assert_eq!(*id_28, 77);
                    assert_eq!(string_30.as_deref(), Some("Station 🌊"));
                }
                NativeJobBody::VisitSystem {
                    id_10,
                    field_14,
                    string_18,
                    id_20,
                    string_28,
                } => {
                    assert_eq!(*id_10, u32::MAX);
                    assert_eq!(*field_14, i32::MIN);
                    assert_eq!(string_18, &None);
                    assert_eq!(*id_20, 77);
                    assert_eq!(string_28.as_deref(), Some("System 🌊"));
                }
                _ => unreachable!(),
            }
            for end in 0..length {
                let mut input = QtReader::new(&bytes[..end]);
                assert_eq!(
                    NativeJob::read_qt7(kind, &mut input).unwrap_err().kind(),
                    io::ErrorKind::UnexpectedEof
                );
                assert_eq!(input.remaining(), end);
            }
        }
    }
}

#[test]
fn cargo_short_overloads_zero_extend_every_bit_pattern_after_manufacture_base() {
    let mut bytes = fixture(9, 0);
    let extra = bytes.len() - 2;
    for word in 0..=u16::MAX {
        bytes[9..11].copy_from_slice(&word.to_be_bytes());
        bytes[extra..].copy_from_slice(&word.to_be_bytes());
        let job = NativeJob::read_qt7(9, &mut QtReader::new(&bytes)).unwrap();
        let NativeJobBody::DeliverCargoManufacture { cargo, field_48 } = job.body else {
            panic!("wrong body");
        };
        assert_eq!(cargo.field_10, u32::from(word));
        assert_eq!(field_48, u32::from(word));
    }
}

#[test]
fn person_mixed_jobs_finish_before_extended_scalars_and_final_nibbles() {
    let mut bytes = vec![0x88, 0x0c, 0x40, 0, 4];
    for kind in [9, 15, 19, 26] {
        bytes.push(kind);
        bytes.extend(fixture(kind, 0));
    }
    bytes.extend(17_i32.to_be_bytes());
    bytes.extend(18_i16.to_be_bytes());
    bytes.extend([0xab, 0xee]);
    let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
    let mut input = QtReader::new(&bytes);
    let report = state
        .receive_suffix(
            &mut input,
            FloatEncoding::TypeWidths,
            std::time::Instant::now,
        )
        .unwrap();
    assert_eq!(report.jobs.unwrap().stop, None);
    assert_eq!(
        state
            .jobs_13b8
            .unwrap()
            .0
            .iter()
            .map(|job| job.kind)
            .collect::<Vec<_>>(),
        [9, 15, 19, 26]
    );
    assert_eq!(state.field_10d0, Some(17));
    assert_eq!(state.field_f58, Some(18));
    assert_eq!(state.field_83d, Some(11));
    assert_eq!(input.read_u8().unwrap(), 0xee);
}
