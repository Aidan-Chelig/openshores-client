use super::*;
use std::time::Instant;

#[test]
fn constructor_subset_preserves_known_omissions_and_does_not_guess_unknowns() {
    let mut state = NativePersonSuffixState::from_default_constructor_subset();
    let initial = state.clone();
    let bytes = [0, 0xee];
    let mut input = QtReader::new(&bytes);
    let report = state
        .receive_suffix(&mut input, FloatEncoding::TypeWidths, || {
            panic!("omitted timer must not start")
        })
        .unwrap();
    assert!(!report.timer_restarted);
    assert_eq!(state, initial);
    assert_eq!(input.read_u8().unwrap(), 0xee);
    assert_eq!(state.identity.string_10f0, Some(None));
    assert_eq!(state.identity.field_1114, None);
    assert_eq!(state.hit_notify, NativeHitNotify::default());
    assert_eq!(state.field_83b, Some(15));
    assert_eq!(state.field_83d, Some(15));
    assert_eq!((state.field_838, state.field_839), (Some(0), Some(0)));
    assert_eq!(state.field_f80, Some(0));
    assert_eq!(state.field_f84, Some(0));
    assert_eq!(state.field_f88, Some(0));
    assert_eq!((state.field_fa8, state.field_fa9), (Some(0), Some(0)));
    assert_eq!(state.field_fb0, Some(0));
    assert_eq!(state.field_10d0, Some(0));
    assert_eq!(state.field_f58, Some(0));
    assert_eq!(state.field_1408, Some(0.));
    assert_eq!((state.field_f60, state.field_f68), (Some(0.), Some(0.)));
    assert_eq!(state.field_10e8, Some(0));
    assert_eq!(state.weight_13a8, Some(0.));
    assert_eq!(state.content_fa0, Some(Vec::new()));
    assert!(state.domain_1018.unwrap().0.is_empty());
    assert!(state.jobs_13b8.unwrap().0.is_empty());
    assert!(state.gear_1390.unwrap().0.is_empty());
    assert!(state.inventory_1458.unwrap().0.is_empty());
    // Observation-only Default is deliberately not changed by this constructor.
    let mut unknown = NativePersonSuffixState::default();
    assert_eq!(
        unknown
            .receive_suffix(
                &mut QtReader::new(&[0]),
                FloatEncoding::TypeWidths,
                Instant::now
            )
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn fresh_masked_updates_use_proven_zero_prior_and_restart_only_on_change() {
    for packed in [0_u16, 1, 0x7fff, 0x8000, 0xffff] {
        let mut state = NativePersonSuffixState::from_default_constructor_subset();
        let now = Instant::now();
        let mut bytes = vec![0x84, 0xc0];
        bytes.extend(packed.to_be_bytes());
        bytes.push(0xee);
        let mut input = QtReader::new(&bytes);
        let report = state
            .receive_suffix(&mut input, FloatEncoding::TypeWidths, || now)
            .unwrap();
        let changed = packed & 0x7fff != 0;
        assert_eq!(report.timer_restarted, changed);
        assert_eq!(state.field_660, Some(packed & 0x7fff));
        assert_eq!(state.field_839, Some((packed >> 15) as u8));
        assert_eq!(state.timer_668, changed.then_some(now));
        assert_eq!(state.state_1130, Some(0x208000));
        assert_eq!(input.read_u8().unwrap(), 0xee);
        let report = state
            .receive_suffix(&mut QtReader::new(&[0]), FloatEncoding::TypeWidths, || {
                panic!("sparse update must not restart timer")
            })
            .unwrap();
        assert!(!report.timer_restarted);
        assert_eq!(state.field_660, Some(packed & 0x7fff));
        assert_eq!(state.state_1130, Some(0x200000));
        assert_eq!(state.field_83d, Some(15));
    }
}
