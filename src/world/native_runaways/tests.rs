use super::*;

#[test]
fn exact_duplicates_are_suppressed_but_shared_parents_and_stale_child_pairs_are_retained() {
    let mut queue = NativeRunaways::default();
    assert!(queue.is_empty());
    for (parent, child) in [(7, 1), (7, 2), (8, 1), (0, 0), (u32::MAX, u32::MAX)] {
        assert!(queue.add(parent, child).unwrap());
        assert!(!queue.add(parent, child).unwrap());
    }
    assert_eq!(queue.len(), 5);
    assert_eq!(queue.children(7), Some(&BTreeSet::from([1, 2])));
    assert_eq!(queue.children(8), Some(&BTreeSet::from([1])));
    assert!(queue.children(9).is_none());
}

#[test]
fn safety_rejection_leaves_all_existing_pairs_unchanged_and_duplicate_still_succeeds() {
    let mut queue = NativeRunaways::with_limit(1).unwrap();
    queue.add(7, 1).unwrap();
    let before = queue.clone();
    assert!(queue.add(8, 1).is_err());
    assert!(queue.add(7, 2).is_err());
    assert_eq!(queue, before);
    assert!(!queue.add(7, 1).unwrap());
    assert_eq!(queue, before);
    assert!(NativeRunaways::with_limit(0).is_err());
    assert!(NativeRunaways::with_limit(MAX_PAIRS + 1).is_err());
}
