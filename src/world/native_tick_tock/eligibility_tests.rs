use super::*;

fn context() -> Eligibility {
    Eligibility {
        field_1c: 0,
        has_owner: true,
        parent_id: 7,
        parent_field_1c: Some(0),
        does_query_environment: false,
        environment_field_34: None,
        capsule_slot_b88: None,
    }
}

#[test]
fn raw_status_ranges_and_parent_gates_match_native_unsigned_comparisons() {
    for field in [i32::MIN, -100, -1, 0, 1, 2, 3, 4, 5, 6, i32::MAX] {
        assert_eq!(
            Eligibility {
                field_1c: field,
                ..context()
            }
            .must_not_tick_tock(),
            field == 2 || field == 3 || field >= 5
        );
    }
    assert!(
        Eligibility {
            has_owner: false,
            ..context()
        }
        .must_not_tick_tock()
    );
    assert!(
        Eligibility {
            parent_id: 0,
            ..context()
        }
        .must_not_tick_tock()
    );
    assert!(
        Eligibility {
            parent_field_1c: None,
            ..context()
        }
        .must_not_tick_tock()
    );
    for field in [i32::MIN, -1, 0, 1, 2, 3, 4, 5, i32::MAX] {
        assert_eq!(
            Eligibility {
                parent_field_1c: Some(field),
                ..context()
            }
            .must_not_tick_tock(),
            field == 2
        );
    }
}

#[test]
fn query_presence_raw_flags_and_capsule_return_have_distinct_gates() {
    for query in [false, true] {
        for flags in [
            None,
            Some(0),
            Some(8),
            Some(0x20),
            Some(0x28),
            Some(0x10),
            Some(u32::MAX),
        ] {
            for capsule in [None, Some(false), Some(true)] {
                let result = Eligibility {
                    does_query_environment: query,
                    environment_field_34: flags,
                    capsule_slot_b88: capsule,
                    ..context()
                }
                .must_not_tick_tock();
                let eligible_query = query && flags.is_none_or(|flags| flags & 0x28 != 0);
                assert_eq!(result, eligible_query && capsule == Some(false));
            }
        }
    }
}
