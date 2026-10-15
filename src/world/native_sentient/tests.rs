use super::*;
use crate::protocol::QtWriter;

fn seed() -> NativeOrderSeed {
    NativeOrderSeed {
        translation: [2., 4., 6.],
        denominator_terms: [1., 0., 0., 1.],
        wall_clock_ms: -17,
    }
}

#[test]
fn all_flags_profiles_and_prior_order_counts_preserve_native_order_and_suffix() {
    for flags in 0..=255_u8 {
        for encoding in [
            FloatEncoding::TypeWidths,
            FloatEncoding::SinglePrecision,
            FloatEncoding::DoublePrecision,
        ] {
            for prior in 0..=2 {
                let mut old = NativeOrder::initialize(seed());
                old.field_20 = Some("retained".into());
                old.time_28 = 99;
                let mut state = NativeSentientState {
                    orders: Some(vec![old; prior]),
                    field_6f0: Some(77),
                    ..Default::default()
                };
                let mut bytes = vec![flags];
                if flags & 1 != 0 {
                    if encoding == FloatEncoding::DoublePrecision {
                        bytes.extend(1.0000000001_f64.to_be_bytes());
                    } else {
                        bytes.extend(1_f32.to_be_bytes());
                    }
                }
                let kind = if flags & 8 != 0 { 0x29 } else { 0xfe };
                if flags & 2 != 0 {
                    bytes.push(kind);
                    bytes.extend(u32::MAX.to_be_bytes());
                    if kind == 0x29 {
                        let mut writer = QtWriter::default();
                        writer.write_string(None).unwrap();
                        bytes.extend(writer.into_bytes());
                    }
                    bytes.push(0x80);
                }
                if flags & 4 != 0 {
                    bytes.extend([0xff, 0, 3]);
                    for (id, value) in
                        [(0x8000_u16, -1_i32), (0xffff, i32::MIN), (0x8000, i32::MAX)]
                    {
                        bytes.extend(id.to_be_bytes());
                        bytes.extend(value.to_be_bytes());
                    }
                }
                let length = bytes.len();
                bytes.push(0xde);
                let mut input = QtReader::new(&bytes);
                let mut calls = 0;
                assert_eq!(
                    state
                        .receive_suffix(&mut input, encoding, || {
                            calls += 1;
                            Ok(seed())
                        })
                        .unwrap(),
                    length
                );
                assert_eq!(calls, usize::from(flags & 2 != 0 && prior != 1));
                assert_eq!(state.field_708, Some(if flags & 1 != 0 { 1. } else { 0. }));
                let orders = state.orders.as_ref().unwrap();
                if flags & 2 != 0 {
                    assert_eq!(orders.len(), 1);
                    assert_eq!(orders[0].kind_30, u32::from(kind));
                    assert_eq!(orders[0].id_1c, u32::MAX);
                    assert_eq!(orders[0].time_28, if prior == 1 { 99 } else { -17 });
                    assert_eq!(orders[0].position, [1., 2., 3.]);
                    assert_eq!(
                        orders[0].field_20.as_deref(),
                        if kind == 0x29 {
                            None
                        } else if prior == 1 {
                            Some("retained")
                        } else {
                            Some("")
                        }
                    );
                } else {
                    assert!(orders.is_empty());
                }
                assert_eq!(state.field_6f0, Some(if flags & 2 != 0 { 128 } else { 77 }));
                assert_eq!(state.skills.is_some(), flags & 4 != 0);
                if let Some(skills) = &state.skills {
                    assert_eq!(skills.0.len(), 2);
                    assert_eq!(skills.0[&0x8000], i32::MAX);
                    assert_eq!(skills.0[&0xffff], i32::MIN);
                }
                assert_eq!(input.read_u8().unwrap(), 0xde);
            }
        }
    }
}

#[test]
fn order_initialization_uses_exact_sum_and_preserves_raw_nonfinite_results() {
    for denominator in [0., 1., 2., -2., f64::INFINITY, f64::NAN] {
        let mut seed = seed();
        seed.denominator_terms = [denominator, 0., 0., 0.];
        let order = NativeOrder::initialize(seed);
        for (actual, raw) in order.position.into_iter().zip(seed.translation) {
            let expected = if denominator == 0. || denominator == 1. {
                raw
            } else {
                raw / denominator
            };
            if expected.is_nan() {
                assert!(actual.is_nan());
            } else {
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
        }
    }
}

#[test]
fn truncated_order_keeps_completed_writes_and_never_receives_skills_early() {
    let mut bytes = vec![7];
    bytes.extend(2_f32.to_be_bytes());
    bytes.push(0x29);
    bytes.extend(0x80000000_u32.to_be_bytes());
    let mut writer = QtWriter::default();
    writer.write_string(Some("order 🦀")).unwrap();
    bytes.extend(writer.into_bytes());
    bytes.push(0xfe);
    bytes.extend([0, 0, 0]); // ignored skill header, zero count
    for end in 0..bytes.len() {
        let mut state = NativeSentientState::default();
        assert!(
            state
                .receive_suffix(
                    &mut QtReader::new(&bytes[..end]),
                    FloatEncoding::TypeWidths,
                    || Ok(seed())
                )
                .is_err()
        );
        if end >= 5 {
            assert_eq!(state.field_708, Some(2.));
        }
        if end >= 6 {
            assert_eq!(state.orders.as_ref().unwrap()[0].kind_30, 0x29);
        }
        if end >= 10 {
            assert_eq!(state.orders.as_ref().unwrap()[0].id_1c, 0x80000000);
        }
        if end < bytes.len() - 3 {
            assert!(state.skills.is_none());
        }
    }
}

#[test]
fn skill_clear_happens_after_header_before_count_and_retains_completed_entries() {
    let mut bytes = vec![0xff, 0, 2];
    bytes.extend(0xffff_u16.to_be_bytes());
    bytes.extend((-17_i32).to_be_bytes());
    bytes.extend(0x8000_u16.to_be_bytes());
    bytes.extend(i32::MAX.to_be_bytes());
    for end in 0..bytes.len() {
        let mut skills = NativeSkillSet([(1, 2)].into());
        assert!(skills.receive(&mut QtReader::new(&bytes[..end])).is_err());
        if end == 0 {
            assert_eq!(skills.0.get(&1), Some(&2));
        } else if end < 9 {
            assert!(skills.0.is_empty());
        } else {
            assert_eq!(skills.0.get(&0xffff), Some(&-17));
            assert_eq!(skills.0.len(), 1);
        }
    }
    let mut skills = NativeSkillSet::default();
    skills.receive(&mut QtReader::new(&bytes)).unwrap();
    assert_eq!(skills.0.len(), 2);
    skills.receive(&mut QtReader::new(&[0, 0, 0])).unwrap();
    assert!(skills.0.is_empty());
}

#[test]
fn skill_counts_use_all_sixteen_bits_and_duplicates_do_not_grow_the_map() {
    for count in [32768_u16, 65535] {
        let mut bytes = vec![0xff];
        bytes.extend(count.to_be_bytes());
        for value in 0..count {
            bytes.extend(0xffff_u16.to_be_bytes());
            bytes.extend(i32::from(value).to_be_bytes());
        }
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        let mut skills = NativeSkillSet::default();
        skills.receive(&mut input).unwrap();
        assert_eq!(skills.0.len(), 1);
        assert_eq!(skills.0[&0xffff], i32::from(count) - 1);
        assert_eq!(input.read_u8().unwrap(), 0xde);
    }
}

#[test]
fn omitted_skills_retain_old_map_and_missing_seed_cannot_invent_an_order() {
    let mut state = NativeSentientState {
        field_708: Some(9.),
        orders: Some(vec![NativeOrder::initialize(seed()); 2]),
        field_6f0: Some(0xfe),
        skills: Some(NativeSkillSet([(1, 2)].into())),
    };
    let error = state
        .receive_suffix(&mut QtReader::new(&[2]), FloatEncoding::TypeWidths, || {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "real matrix unavailable",
            ))
        })
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    assert_eq!(state.field_708, Some(0.));
    assert!(state.orders.as_ref().unwrap().is_empty());
    state
        .receive_suffix(&mut QtReader::new(&[0]), FloatEncoding::TypeWidths, || {
            panic!("not needed")
        })
        .unwrap();
    assert_eq!(state.field_6f0, Some(0xfe));
    assert_eq!(state.skills.as_ref().unwrap().0.get(&1), Some(&2));
}
