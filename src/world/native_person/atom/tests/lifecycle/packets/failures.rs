use super::*;

#[test]
fn truncated_and_explicitly_unsupported_packets_never_report_post_update_success() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let bytes = complete(profile);
        for end in 0..bytes.len() {
            let mut h = Harness::new(profile);
            assert!(h.receive(&bytes[..end]).is_err(), "prefix {end}");
            assert!(h.hooks.reports.is_empty());
            assert!(h.hooks.post.is_empty());
        }
    }
    let mut h = Harness::new(FloatEncoding::TypeWidths);
    h.hooks.reject_construction = true;
    assert_eq!(
        h.receive(&complete(h.profile)).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert!(h.lookup.find_atom(7).is_none());
    assert!(h.connector.0.is_empty());
    assert!(!h.people.get(90).unwrap().base.owner_attached);
    let mut h = Harness::new(FloatEncoding::TypeWidths);
    assert_eq!(
        h.receive(&packet(123, &[0, 0, 0, 1, 0]))
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    ); // configuration callback rejects before the nested DNA body
    assert!(h.hooks.reports.is_empty());
    assert!(h.hooks.post.is_empty());
}

#[test]
fn unrelated_opcodes_and_other_retained_classes_are_not_reinterpreted_as_person() {
    let mut h = Harness::new(FloatEncoding::TypeWidths);
    assert_eq!(h.receive(&[0xff]).unwrap(), None);
    let mut other = complete(h.profile);
    other[0] = 0x16;
    assert_eq!(
        h.receive(&other).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert!(h.people.is_empty());
    assert!(h.hooks.events.is_empty());
    h.lookup = NativeRecoveryLookup::new([(7, 50)], []).unwrap();
    assert_eq!(
        h.receive(&complete(h.profile)).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(h.lookup.find_atom(7), Some(50));
    assert_eq!(h.hooks.events, [(90, "construct")]);
    assert!(h.people.get(90).is_some()); // completed construction survives Rust error
    assert!(!h.people.get(90).unwrap().base.owner_attached);
    assert!(h.owner.assigned.is_empty());
    assert!(h.connector.0.is_empty());
    assert!(h.hooks.reports.is_empty());
    assert!(h.hooks.post.is_empty());
}
