use super::*;

fn bytes(flags: u8) -> Vec<u8> {
    let mut bytes = vec![flags, 0xab, 0xcd];
    for bit in [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80] {
        if flags & bit == 0 {
            continue;
        }
        match bit {
            2 => {
                bytes.push(255);
                bytes.extend(2.5_f32.to_be_bytes());
                bytes.extend(3.5_f32.to_be_bytes());
            }
            4 => bytes.push(0xff),
            0x40 | 0x80 => {
                bytes.push(199);
                bytes.extend(4.5_f32.to_be_bytes());
            }
            _ => {
                for value in [1.25_f32, -2.5, 3.75] {
                    bytes.extend(value.to_be_bytes());
                }
            }
        }
    }
    bytes
}
#[test]
fn every_flag_combination_and_truncation_has_exact_boundaries() {
    for flags in 0..=255 {
        let mut payload = bytes(flags);
        let length = payload.len();
        payload.push(0xde);
        let original = NativeEffect::from_base_receive_temporary();
        let mut state = original.clone();
        let mut input = QtReader::new(&payload);
        assert_eq!(state.receive(&mut input, 0x12340000, 0).unwrap(), length);
        assert_eq!(input.remaining_bytes(), &[0xde]);
        assert_eq!(state.time_70, 0x1234abcd);
        assert_eq!(state.field_08, if flags & 2 != 0 { 2.5 } else { 1. });
        assert_eq!(state.field_64, if flags & 2 != 0 { 255 } else { 0 });
        assert_eq!(state.field_a8, if flags & 4 != 0 { 127 } else { 0 });
        assert_eq!(state.field_68, flags & 4 != 0);
        assert_eq!(state.field_ac, if flags & 0x40 != 0 { 199 } else { 0 });
        assert_eq!(state.field_b4, if flags & 0x80 != 0 { 199 } else { 0 });
        for truncated in 0..length {
            let mut state = original.clone();
            let mut input = QtReader::new(&payload[..truncated]);
            assert!(state.receive(&mut input, 0, 0).is_err());
            assert_eq!(state, original);
            assert_eq!(input.remaining(), truncated);
        }
    }
}
#[test]
fn sparse_reuse_retains_vectors_but_zeros_final_pairs() {
    let mut state = NativeEffect::from_base_receive_temporary();
    state
        .receive(&mut QtReader::new(&bytes(255)), 0, 0)
        .unwrap();
    state.receive(&mut QtReader::new(&bytes(0)), 0, 0).unwrap();
    assert_eq!(state.vector_28, [1.25, -2.5, 3.75]);
    assert_eq!(state.vector_90, [1.25, -2.5, 3.75]);
    assert_eq!(state.field_08, 2.5);
    assert_eq!(
        (
            state.field_ac,
            state.field_b0,
            state.field_b4,
            state.field_b8
        ),
        (0, 0., 0, 0.)
    );
}
#[test]
fn clock_wrap_and_subset_float_comparison_follow_native_operations() {
    let mut state = NativeEffect::from_base_receive_temporary();
    state
        .receive(&mut QtReader::new(&[0, 0, 1]), i64::MAX, 1)
        .unwrap();
    assert_eq!(state.time_70, i64::MIN + 1);
    state
        .receive(&mut QtReader::new(&[0, 255, 255]), 0x10000, 0)
        .unwrap();
    assert_eq!(state.time_70, 0x1ffff); // not nearest/past epoch
    let mut other = state.clone();
    other.field_b0 = 99.;
    other.vector_90[0] = f64::NAN;
    assert!(state.matches_append_key(&other));
    other.vector_28[0] = -0.;
    assert!(state.matches_append_key(&other));
    other.field_08 = f32::NAN;
    assert!(!other.matches_append_key(&other));
}
