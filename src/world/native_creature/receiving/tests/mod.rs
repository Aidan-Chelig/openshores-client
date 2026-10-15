use super::*;
mod context;
use context::{Event, Trace};
mod fixtures;
use fixtures::*;
mod partial;
mod scalars;

#[test]
fn unit_and_all_creature_flag_bits_are_received_in_order_with_real_nested_codecs() {
    for flags in 0..=u16::MAX {
        let profile = FloatEncoding::TypeWidths;
        let mut bytes = fixture(flags, profile, 0xff);
        let length = bytes.len();
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        let mut state = NativeCreatureReceiveState::default();
        let mut context = Trace {
            now: 123456789,
            ..Trace::default()
        };
        let result = state
            .receive_qt7(&mut input, profile, &mut context)
            .unwrap();
        assert_eq!(result.flags, flags);
        assert_eq!(result.unit.flags, 7);
        assert_eq!(result.unit.consumed_bytes, unit(profile).len());
        assert_eq!(result.consumed_bytes, length);
        assert_eq!(input.read_u8().unwrap(), 0xde);
        assert_eq!(
            context.events,
            expected_events(flags, 0xff, true, context.now)
        );
        assert_eq!(state.unit.fields.field_3b0, Some(None));
        assert_eq!(state.unit.fields.field_3a8, Some(-5));
        assert_eq!(state.unit.fields.field_39c, Some(0xfe));
        assert_eq!(state.unit.fields.vector_330, Some([1., -0., -7.25]));
        assert_eq!(state.unit.conditions.as_ref().unwrap().0.len(), 1);
        assert_eq!(state.unit.conditions.as_ref().unwrap().0[0].time_08, -999);
        let body = state.body.as_ref().unwrap();
        assert_eq!(body.hit_points, if flags & 4 != 0 { -30 } else { 0 });
        assert_eq!(
            body.received_dna,
            if flags & 1 != 0 {
                Some(dna(flags & 0x100 != 0).1)
            } else {
                None
            }
        );
        assert_eq!(
            body.received_configuration,
            if flags & 1 != 0 { Some(0xff) } else { None }
        );
        assert_eq!(
            body.timestamp_ms,
            if flags & 0x40 != 0 {
                context.now - 86_400_000
            } else {
                context.now
            }
        );
        assert_eq!(
            state.scalar_1e0,
            if flags & 2 != 0 { Some(1.) } else { None }
        );
        assert_eq!(state.field_496, Some(0xc3));
        assert_eq!(state.field_494, Some(if flags & 0x20 != 0 { 4 } else { 0 }));
        assert_eq!(state.field_495, Some(0xfe));
        assert_eq!(
            state.field_5b8,
            if flags & 0x200 != 0 { Some(255) } else { None }
        );
        assert_eq!(
            state.field_5bc,
            if flags & 0x200 != 0 { Some(128) } else { None }
        );
        assert_eq!(state.item_698.is_some(), flags & 0x80 != 0);
        if let Some(item) = state.item_698.as_ref() {
            assert_eq!(item.extension.type_id(), 1);
            assert_eq!(item.common.commodity, 1);
            assert_eq!(item.common.field_20(), 1);
        }
        assert_eq!(context.game.is_some(), flags & 0x10 != 0);
    }
}

#[test]
fn profiles_configuration_bytes_and_repeated_sparse_updates_preserve_distinct_rules() {
    for profile in PROFILES {
        for packed in 0..=255 {
            let mut state = NativeCreatureReceiveState::default();
            let mut context = Trace::default();
            for flags in [0x2ff, 0, 0x3ff, 0x100, 0] {
                let old = state.clone();
                context.events.clear();
                let created = old.body.as_ref().is_none_or(|body| {
                    body.kind
                        != if flags & 0x100 != 0 {
                            NativeCreatureBodyKind::Animalia
                        } else {
                            NativeCreatureBodyKind::DpBody
                        }
                });
                let bytes = fixture(flags, profile, packed);
                assert_eq!(
                    state
                        .receive_qt7(&mut QtReader::new(&bytes), profile, &mut context)
                        .unwrap()
                        .consumed_bytes,
                    bytes.len()
                );
                assert_eq!(context.events, expected_events(flags, packed, created, 0));
                let body = state.body.as_ref().unwrap();
                if flags & 1 != 0 {
                    assert_eq!(body.received_configuration, Some(packed));
                } else if !created {
                    assert_eq!(body.received_dna, old.body.as_ref().unwrap().received_dna);
                } else {
                    assert_eq!(body.received_dna, None);
                    assert_eq!(body.hit_points, 0);
                }
                if flags & 0x200 == 0 {
                    assert_eq!(state.field_5b8, old.field_5b8);
                    assert_eq!(state.field_5bc, old.field_5bc);
                }
                if flags & 0x20 == 0 {
                    assert_eq!(state.field_494, Some(0));
                }
                if flags & 0x80 == 0 {
                    assert!(state.item_698.is_none());
                }
                if flags & 2 == 0 {
                    assert_eq!(state.scalar_1e0, old.scalar_1e0);
                }
            }
        }
    }
}
