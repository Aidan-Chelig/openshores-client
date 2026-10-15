use super::*;

#[test]
fn signed_hit_points_use_only_the_native_lower_clamp_in_both_body_modes() {
    for kind_flag in [0_u16, 0x100] {
        for raw in i16::MIN..=i16::MAX {
            let mut bytes = vec![0];
            bytes.extend((kind_flag | 4).to_be_bytes());
            bytes.extend(raw.to_be_bytes());
            bytes.push(0xfe);
            let mut state = NativeCreatureReceiveState::default();
            let mut context = Trace::default();
            state
                .receive_qt7(
                    &mut QtReader::new(&bytes),
                    FloatEncoding::TypeWidths,
                    &mut context,
                )
                .unwrap();
            let body = state.body.as_ref().unwrap();
            assert_eq!(body.hit_points, raw.max(-30));
            assert!(context.events.contains(&Event::Hp(body.kind, raw.max(-30))));
        }
    }
}

#[test]
fn scalar_float_overload_keeps_nonfinite_and_signed_zero_for_the_virtual_setter() {
    for profile in PROFILES {
        for raw in [f64::MAX, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.] {
            let mut bytes = vec![0, 0, 2];
            float(&mut bytes, raw, profile);
            bytes.push(0xff);
            let mut state = NativeCreatureReceiveState::default();
            let mut context = Trace::default();
            state
                .receive_qt7(&mut QtReader::new(&bytes), profile, &mut context)
                .unwrap();
            let value = state.scalar_1e0.unwrap();
            let Event::Scalar(observed) = context.events[1] else {
                panic!("setter is before old console sampling")
            };
            if raw.is_nan() {
                assert!(value.is_nan());
                assert!(observed.is_nan());
            } else {
                assert_eq!(value.to_bits(), (raw as f32).to_bits());
                assert_eq!(observed.to_bits(), value.to_bits());
            }
        }
    }
}

#[test]
fn wall_clock_is_sampled_after_item_before_final_bytes_and_day_subtraction_wraps() {
    let profile = FloatEncoding::TypeWidths;
    for now in [i64::MIN, 0, i64::MAX] {
        let bytes = fixture(0x2c0, profile, 0);
        let mut state = NativeCreatureReceiveState::default();
        let mut context = Trace {
            now,
            ..Trace::default()
        };
        state
            .receive_qt7(&mut QtReader::new(&bytes), profile, &mut context)
            .unwrap();
        assert_eq!(
            state.body.as_ref().unwrap().timestamp_ms,
            now.wrapping_sub(86_400_000)
        );
        let expected = expected_events(0x2c0, 0, true, now);
        let clock_index = expected
            .iter()
            .position(|event| *event == Event::Clock)
            .unwrap();
        let mut context = Trace {
            fail_at: Some(clock_index),
            now,
            ..Trace::default()
        };
        let mut state = NativeCreatureReceiveState::default();
        let mut input = QtReader::new(&bytes);
        assert_eq!(
            state
                .receive_qt7(&mut input, profile, &mut context)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(input.remaining_bytes(), &[0xff, 0x80]);
        assert!(state.item_698.is_some());
        assert_eq!(state.field_5b8, None);
        assert_eq!(state.field_5bc, None);
    }
}
