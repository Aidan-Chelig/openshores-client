use super::*;

#[test]
fn content_ids_use_byte_count_raw_byte_values_and_clear_only_when_flagged() {
    for packed in 0..=255_u8 {
        for count in 0..=255_u8 {
            let mut bytes = vec![0x20, 0xa1, packed];
            if packed & 0x80 != 0 {
                bytes.push(count);
                bytes.extend(0..count);
            }
            bytes.extend([0xfe, 0xee]);
            let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
            state.content_fa0 = Some(vec![99, 88]);
            let mut input = QtReader::new(&bytes);
            let _ = state
                .receive_suffix(&mut input, FloatEncoding::TypeWidths, Instant::now)
                .unwrap();
            assert_eq!(state.field_fa8, Some(0xa1));
            assert_eq!(state.field_fa9, Some(packed & 15));
            assert_eq!(
                state.content_fa0.unwrap(),
                if packed & 0x80 != 0 {
                    (0..count).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            );
            assert_eq!(state.field_fb0, Some(0xfe));
            assert_eq!(input.read_u8().unwrap(), 0xee);
        }
    }
}

#[test]
fn raw_packed_bytes_preserve_native_masks_and_inventory_failure_position() {
    for packed in 0..=255_u8 {
        let mut bytes = vec![0x10, packed];
        bytes.extend(17_i32.to_be_bytes());
        let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
        let _ = state
            .receive_suffix(
                &mut QtReader::new(&bytes),
                FloatEncoding::TypeWidths,
                Instant::now,
            )
            .unwrap();
        assert_eq!(state.field_f84, Some(packed >> 7));
        assert_eq!(state.field_f88, Some(u32::from((packed >> 3) & 3)));
        let mut state = NativePersonSuffixState::with_prior_fields(0, 0x8000);
        let bytes = [8, packed, 0xab, 0xee];
        let mut input = QtReader::new(&bytes);
        let result = state.receive_suffix(&mut input, FloatEncoding::TypeWidths, Instant::now);
        assert_eq!(state.field_838, Some(packed & 0x3f));
        assert_eq!(state.state_1130, Some(0));
        if packed & 0x80 != 0 {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
            assert_eq!(input.remaining_bytes(), [0xab, 0xee]);
        } else {
            let _ = result.unwrap();
            assert_eq!(state.field_83d, (packed & 0x40 != 0).then_some(0xb));
            assert_eq!(state.field_83b, (packed & 0x40 != 0).then_some(0xa));
            assert_eq!(input.remaining(), if packed & 0x40 != 0 { 1 } else { 2 });
        }
    }
}

#[test]
fn content_failure_retains_completed_bytes_and_earlier_fields() {
    let bytes = [0x20, 0xa1, 0x8f, 3, 0, 0x80, 0xff, 0xfe];
    for end in 1..bytes.len() {
        let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
        state.content_fa0 = Some(vec![99, 88]);
        assert_eq!(
            state
                .receive_suffix(
                    &mut QtReader::new(&bytes[..end]),
                    FloatEncoding::TypeWidths,
                    Instant::now
                )
                .unwrap_err()
                .kind(),
            io::ErrorKind::UnexpectedEof
        );
        if end >= 3 {
            assert_eq!(state.content_fa0.as_deref(), Some(&bytes[4..end.max(4)]));
            assert_eq!(state.field_fa9, Some(15));
        } else {
            assert_eq!(state.content_fa0, Some(vec![99, 88]));
        }
        assert_eq!(state.field_fb0, None);
        assert_eq!(state.state_1130, Some(0));
    }
}
