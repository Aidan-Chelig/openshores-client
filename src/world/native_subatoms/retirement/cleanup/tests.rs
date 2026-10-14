use super::*;

#[derive(Default)]
struct Context {
    elapsed: BTreeMap<u32, i64>,
    queued: BTreeSet<u32>,
    null: bool,
    same: bool,
    parent: Option<u32>,
    alternate: Option<u32>,
    events: Vec<(u32, &'static str)>,
}
impl NativeRetirementCleanupContext for Context {
    fn elapsed_reference_millis(&mut self, handle: u32, _: Instant, _: Instant) -> i64 {
        self.events.push((handle, "age"));
        self.elapsed[&handle]
    }
    fn in_tick_tock_queue(&mut self, handle: u32, retired: &NativeRetiredAtoms) -> bool {
        assert!(retired.started_at(handle).is_some());
        self.events.push((handle, "queue"));
        self.queued.contains(&handle)
    }
    fn is_null_object(&mut self, handle: u32) -> bool {
        self.events.push((handle, "null"));
        self.null
    }
    fn find_same_object_by_id(&mut self, handle: u32) -> bool {
        self.events.push((handle, "same"));
        self.same
    }
    fn containing_parent_id(&mut self, handle: u32) -> Option<u32> {
        self.events.push((handle, "parent"));
        self.parent
    }
    fn alternate_map_id(&mut self, handle: u32) -> Option<u32> {
        self.events.push((handle, "alternate"));
        self.alternate
    }
    fn destroy(&mut self, handle: u32, remaining: usize, retired: &NativeRetiredAtoms) {
        assert!(retired.started_at(handle).is_none());
        assert_eq!(remaining, retired.len());
        self.events.push((handle, "destroy"));
    }
}
fn store(handles: &[u32], now: Instant) -> NativeRetiredAtoms {
    NativeRetiredAtoms {
        started: handles.iter().map(|&h| (h, now)).collect(),
    }
}

#[test]
fn explicit_order_restarts_queued_and_deletes_only_first_unqueued_old_object() {
    let start = Instant::now();
    let now = start + std::time::Duration::from_secs(30);
    let mut retired = store(&[1, 2, 3, 4], start);
    let mut ctx = Context {
        elapsed: BTreeMap::from([(1, 30000), (2, 30000), (3, 19999), (4, 30000)]),
        queued: BTreeSet::from([4]),
        ..Default::default()
    };
    let result = retired
        .delete_one_in_order(now, &[3, 4, 2, 1], false, &mut ctx)
        .unwrap();
    assert_eq!(result.remaining, 3);
    assert_eq!(result.restarted, [4]);
    assert_eq!(
        result.selected,
        Some((2, NativeRetirementDisposition::Destroyed))
    );
    assert_eq!(retired.started_at(4), Some(now));
    assert_eq!(retired.started_at(1), Some(start));
    assert_eq!(
        ctx.events,
        [
            (3, "age"),
            (4, "age"),
            (4, "queue"),
            (2, "age"),
            (2, "queue"),
            (2, "null"),
            (2, "destroy")
        ]
    );
}

#[test]
fn signed_boundary_and_empty_store_skip_unneeded_getters() {
    let now = Instant::now();
    for age in [i64::MIN, -1, 0, 19999, 20000, i64::MAX] {
        let mut retired = store(&[0], now);
        let mut ctx = Context {
            elapsed: BTreeMap::from([(0, age)]),
            ..Default::default()
        };
        let result = retired
            .delete_one_in_order(now, &[0], false, &mut ctx)
            .unwrap();
        assert_eq!(result.selected.is_some(), age >= 20000);
        assert_eq!(ctx.events.len(), if age >= 20000 { 4 } else { 1 });
    }
    let mut ctx = Context::default();
    assert_eq!(
        NativeRetiredAtoms::default()
            .delete_one_in_order(now, &[], true, &mut ctx)
            .unwrap()
            .remaining,
        0
    );
    assert!(ctx.events.is_empty());
}

#[test]
fn guards_erase_tracking_without_requeue_and_preserve_native_zero_sentinel() {
    use NativeRetirementDisposition::*;
    let now = Instant::now();
    for (null, same, parent, alternate, disposition, calls) in [
        (true, true, Some(9), Some(8), NullObject, vec!["null"]),
        (
            false,
            true,
            Some(9),
            Some(8),
            StillRegisteredById,
            vec!["null", "same"],
        ),
        (
            false,
            false,
            Some(9),
            Some(8),
            StillInParent(9),
            vec!["null", "same", "parent"],
        ),
        (
            false,
            false,
            Some(0),
            Some(8),
            StillRegisteredUnderOtherId(8),
            vec!["null", "same", "parent", "alternate"],
        ),
        (
            false,
            false,
            Some(0),
            Some(0),
            Destroyed,
            vec!["null", "same", "parent", "alternate", "destroy"],
        ),
        (
            false,
            false,
            None,
            None,
            Destroyed,
            vec!["null", "same", "parent", "alternate", "destroy"],
        ),
    ] {
        let mut retired = store(&[1, 2], now);
        let mut ctx = Context {
            elapsed: BTreeMap::from([(1, 20000)]),
            null,
            same,
            parent,
            alternate,
            ..Default::default()
        };
        let result = retired
            .delete_one_in_order(now, &[1, 2], true, &mut ctx)
            .unwrap();
        assert_eq!(result.selected, Some((1, disposition)));
        assert_eq!(result.remaining, 1);
        assert!(retired.started_at(1).is_none());
        assert_eq!(
            ctx.events[2..].iter().map(|(_, e)| *e).collect::<Vec<_>>(),
            calls
        );
    }
}

#[test]
fn malformed_order_is_atomic_before_callbacks_or_timer_changes() {
    let now = Instant::now();
    for order in [&[1][..], &[1, 1], &[1, 99], &[1, 2, 3]] {
        let mut retired = store(&[1, 2], now);
        let mut ctx = Context::default();
        assert!(
            retired
                .delete_one_in_order(now, order, true, &mut ctx)
                .is_err()
        );
        assert_eq!(retired.len(), 2);
        assert_eq!(retired.started_at(1), Some(now));
        assert!(ctx.events.is_empty());
    }
}

#[test]
fn all_old_queued_entries_restart_without_selecting_or_running_guards() {
    let start = Instant::now();
    let now = start + std::time::Duration::from_secs(30);
    let mut retired = store(&[1, 2], start);
    let mut ctx = Context {
        elapsed: BTreeMap::from([(1, 20000), (2, 30000)]),
        queued: BTreeSet::from([1, 2]),
        ..Default::default()
    };
    let result = retired
        .delete_one_in_order(now, &[2, 1], true, &mut ctx)
        .unwrap();
    assert_eq!(result.remaining, 2);
    assert_eq!(result.selected, None);
    assert_eq!(result.restarted, [2, 1]);
    assert_eq!(retired.started_at(1), Some(now));
    assert_eq!(retired.started_at(2), Some(now));
    assert_eq!(
        ctx.events,
        [(2, "age"), (2, "queue"), (1, "age"), (1, "queue")]
    );
}
