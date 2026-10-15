use super::*;

#[test]
fn dp_words_and_boolean_contribute_but_animation_presence_and_speed_do_not() {
    for index in 0..10 {
        for byte in 0..=255_u32 {
            for mode in [false, true] {
                for present in [false, true] {
                    let mut value = NativeDpBodyPoseState::default();
                    let mut fields = [0; 10];
                    fields[index] = byte;
                    let mut calls = 0;
                    assert_eq!(
                        value
                            .apply(fields, mode, -0., present, |state, speed| {
                                calls += 1;
                                assert_eq!(state.fields_3c, fields);
                                assert_eq!(state.field_6c, mode);
                                assert_eq!(speed.to_bits(), (-0_f32).to_bits());
                                Ok(())
                            })
                            .unwrap(),
                        byte != 0 || mode
                    );
                    assert_eq!(calls, usize::from(present));
                    assert!(
                        !value
                            .apply(fields, mode, f32::NAN, present, |state, speed| {
                                calls += 1;
                                assert_eq!(state.fields_3c, fields);
                                assert!(speed.is_nan());
                                Ok(())
                            })
                            .unwrap()
                    );
                    assert_eq!(calls, 2 * usize::from(present));
                    assert!(
                        value
                            .apply(fields, !mode, 1., false, |_, _| panic!(
                                "empty model data must not animate"
                            ))
                            .unwrap()
                    );
                }
            }
        }
    }
}

#[test]
fn animal_final_pose_normalizes_after_change_detection_and_always_animates() {
    for raw in (0..=255).chain([0x7fff_ffff, 0x8000_0000, u32::MAX]) {
        let mut fields = [0; 10];
        fields[9] = raw;
        let mut value = NativeAnimaliaPoseState::default();
        let mut calls = 0;
        let invalid = !(0x8b..=0x9f).contains(&raw);
        let result = value
            .apply(fields, true, f32::INFINITY, |state| {
                calls += 1;
                assert_eq!(state.fields_6c[9], if invalid { 0x8c } else { raw });
                assert!(state.field_40);
                assert_eq!(state.field_94, f32::INFINITY);
                Ok(())
            })
            .unwrap();
        assert_eq!(result.changed, raw != 0);
        assert_eq!(result.normalized_final_word, invalid.then_some(raw));
        let result = value
            .apply(fields, false, f32::NEG_INFINITY, |state| {
                calls += 1;
                assert!(!state.field_40);
                assert_eq!(state.field_94, f32::NEG_INFINITY);
                Ok(())
            })
            .unwrap();
        assert_eq!(result.changed, invalid && raw != 0x8c);
        assert_eq!(calls, 2);
    }
}

#[test]
fn animal_every_word_changes_and_mode_or_speed_alone_never_does() {
    let mut baseline = [0; 10];
    baseline[9] = 0x8c;
    for index in 0..10 {
        for byte in 0..=255_u32 {
            let mut value = NativeAnimaliaPoseState {
                fields_6c: baseline,
                field_40: false,
                field_94: 0.,
            };
            let mut fields = baseline;
            fields[index] = byte;
            let result = value.apply(fields, true, -0., |_| Ok(())).unwrap();
            assert_eq!(result.changed, byte != baseline[index]);
            assert_eq!(value.field_94.to_bits(), (-0_f32).to_bits());
        }
    }
    let mut value = NativeAnimaliaPoseState {
        fields_6c: baseline,
        field_40: false,
        field_94: 0.,
    };
    for speed in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0., 0., 1.] {
        for mode in [false, true] {
            let result = value
                .apply(baseline, mode, speed, |state| {
                    assert_eq!(state.field_40, mode);
                    assert_eq!(state.field_94.to_bits(), speed.to_bits());
                    Ok(())
                })
                .unwrap();
            assert!(!result.changed);
            assert_eq!(result.normalized_final_word, None);
        }
    }
}

#[test]
fn animation_rejection_is_explicit_and_keeps_prior_pose_data_mutations() {
    let fields = [u32::MAX; 10];
    let mut dp = NativeDpBodyPoseState::default();
    assert_eq!(
        dp.apply(fields, true, 1., true, |_, _| Err(
            io::ErrorKind::Unsupported.into()
        ))
        .unwrap_err()
        .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(dp.fields_3c, fields);
    assert!(dp.field_6c);
    let mut animal = NativeAnimaliaPoseState::default();
    assert_eq!(
        animal
            .apply(fields, true, 7., |_| Err(io::ErrorKind::Unsupported.into()))
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    let mut normalized = fields;
    normalized[9] = 0x8c;
    assert_eq!(animal.fields_6c, normalized);
    assert!(animal.field_40);
    assert_eq!(animal.field_94, 7.);
}
