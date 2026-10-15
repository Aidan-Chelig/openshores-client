use super::*;

#[test]
fn selected_galaxy_location_item_receives_the_creature_float_profile() {
    for profile in PROFILES {
        let mut bytes = unit(profile);
        bytes.extend(0x80_u16.to_be_bytes());
        bytes.push(0xfa);
        bytes.extend([10, 0, 0, 3, 1]);
        bytes.extend(88_u32.to_be_bytes());
        for value in [1.25_f64, -0.0, -99.5] {
            if profile == FloatEncoding::SinglePrecision {
                bytes.extend((value as f32).to_be_bytes());
            } else {
                bytes.extend(value.to_be_bytes());
            }
        }
        bytes.extend(u32::MAX.to_be_bytes());
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        let mut state = NativeCreatureReceiveState::default();
        state
            .receive_qt7(&mut input, profile, &mut Trace::default())
            .unwrap();
        let crate::world::native_item::NativeItemExtension::GalaxyLocation(location) =
            state.item_698.unwrap().extension
        else {
            panic!("wrong selected item");
        };
        assert_eq!(location.id_4c, 88);
        assert_eq!(
            location.point_50_bits,
            [1.25_f64, -0.0, -99.5].map(f64::to_bits)
        );
        assert_eq!(location.string_68, None);
        assert_eq!(input.read_u8().unwrap(), 0xde);
    }
}

#[test]
fn every_truncated_combined_unit_creature_prefix_fails_and_does_not_run_later_effects() {
    for profile in PROFILES {
        for flags in [0x2ff, 0x3ff, 0, 0x100] {
            let bytes = fixture(flags, profile, 0xfe);
            let complete = expected_events(flags, 0xfe, true, 0);
            for end in 0..bytes.len() {
                let mut state = NativeCreatureReceiveState::default();
                let mut context = Trace::default();
                assert!(
                    state
                        .receive_qt7(&mut QtReader::new(&bytes[..end]), profile, &mut context)
                        .is_err()
                );
                assert!(complete.starts_with(&context.events));
                if end < unit(profile).len() + 2 {
                    assert!(context.events.is_empty());
                    assert!(state.body.is_none());
                }
                if end >= unit(profile).len() {
                    assert_eq!(state.unit.conditions.as_ref().unwrap().0[0].time_08, -999);
                }
                assert_eq!(state.field_495, None); // final local seat effect only after complete wire tail
            }
        }
    }
}

#[test]
fn explicit_context_rejection_aborts_at_that_effect_without_guessing_a_suffix() {
    for flags in [0x2ff, 0x3ff] {
        let profile = FloatEncoding::TypeWidths;
        let bytes = fixture(flags, profile, 0xff);
        let expected = expected_events(flags, 0xff, true, 0);
        for stop in 0..expected.len() {
            let mut context = Trace {
                fail_at: Some(stop),
                ..Trace::default()
            };
            let mut state = NativeCreatureReceiveState::default();
            let error = state
                .receive_qt7(&mut QtReader::new(&bytes), profile, &mut context)
                .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::Unsupported);
            assert_eq!(context.events, expected[..=stop]);
            assert_eq!(state.field_495, None);
        }
    }
}

#[test]
fn item_omission_null_factory_and_unsupported_body_never_preserve_the_old_selection() {
    let profile = FloatEncoding::TypeWidths;
    let mut state = NativeCreatureReceiveState::default();
    let mut context = Trace::default();
    let bytes = fixture(0x80, profile, 0);
    state
        .receive_qt7(&mut QtReader::new(&bytes), profile, &mut context)
        .unwrap();
    assert!(state.item_698.is_some());
    let mut bytes = unit(profile);
    bytes.extend(0x80_u16.to_be_bytes());
    bytes.push(0xfa);
    let type_offset = bytes.len();
    bytes.extend([0, 0xde]);
    let mut input = QtReader::new(&bytes);
    state
        .receive_qt7(&mut input, profile, &mut context)
        .unwrap();
    assert!(state.item_698.is_none());
    assert_eq!(input.read_u8().unwrap(), 0xde);
    for unsupported in (1..=255).filter(|kind| ![1, 6, 7, 8, 9, 10, 12, 17].contains(kind)) {
        let mut bytes = bytes.clone();
        bytes[type_offset] = unsupported;
        context.events.clear();
        let mut input = QtReader::new(&bytes);
        assert_eq!(
            state
                .receive_qt7(&mut input, profile, &mut context)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(input.remaining_bytes(), &[unsupported, 0xde]);
        assert!(state.item_698.is_none());
        assert!(!context.events.contains(&Event::Clock));
    }
}

#[test]
fn dp_configuration_effect_occurs_before_a_truncated_dna_and_bad_console_fails_explicitly() {
    let profile = FloatEncoding::TypeWidths;
    let mut bytes = unit(profile);
    bytes.extend([0, 1, 4, 0, 0]);
    let mut state = NativeCreatureReceiveState::default();
    let mut context = Trace::default();
    assert!(
        state
            .receive_qt7(&mut QtReader::new(&bytes), profile, &mut context)
            .is_err()
    );
    assert_eq!(
        context.events,
        vec![
            Event::Select(NativeCreatureBodyKind::DpBody, true),
            Event::Configure([1, 1])
        ]
    );
    assert_eq!(state.body.as_ref().unwrap().dp_configuration, Some([1, 1]));
    assert_eq!(state.body.as_ref().unwrap().received_dna, None);
    let mut bytes = unit(profile);
    bytes.extend([0, 0x10, 0xff, 0xde]);
    let mut input = QtReader::new(&bytes);
    assert_eq!(
        state
            .receive_qt7(&mut input, profile, &mut context)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(input.remaining_bytes(), &[0xde]);
}
