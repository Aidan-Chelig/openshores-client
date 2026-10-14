use super::*;
use crate::world::native_prefix::NativeIdentityPrefix;
#[test]
fn concrete_item_receives_real_layers_and_sparse_body_without_owner_defaults() {
    let mut atom = NativeItemAtom::from_default_item_constructor();
    atom.base.apply_received_identity(NativeIdentityPrefix {
        id: 7,
        timestamp_ms: -123,
    });
    let mut input = QtReader::new(&[0x70, 1, 0, 0, 7, 2, 2, 2, 0x60, 0xde]);
    atom.receive_base(
        &mut input,
        |_, _| panic!("omitted position"),
        || (0x10000, 0),
    )
    .unwrap();
    assert_eq!(atom.receive_known_body(&mut input).unwrap(), Some(1));
    assert_eq!(input.remaining_bytes(), &[0xde]);
    assert_eq!(atom.base.field_1c, 0);
    assert!(!atom.base.owner_attached);
    assert_eq!(atom.received.effects.effects()[0].time_70, 0x10007);
    assert_eq!(atom.received.controllers.entries().len(), 2);
    assert_eq!(atom.item.ownership_id, Some(1));
    assert_eq!(atom.item.value_present, Some(false));
    atom.receive_known_body(&mut QtReader::new(&[0])).unwrap();
    assert_eq!(atom.item.ownership_id, Some(1));
    assert_eq!(atom.item.field_39c, Some(false));
}
#[test]
fn unsupported_and_malformed_bodies_do_not_claim_success_or_erase_observed_state() {
    let mut atom = NativeItemAtom::from_default_item_constructor();
    atom.receive_known_body(&mut QtReader::new(&[0x60]))
        .unwrap();
    let before = atom.item.clone();
    let mut unsupported = QtReader::new(&[1, 249, 0xde]);
    assert_eq!(atom.receive_known_body(&mut unsupported).unwrap(), None);
    assert_eq!(unsupported.remaining_bytes(), &[1, 249, 0xde]);
    assert_eq!(atom.item, before);
    let mut malformed = QtReader::new(&[0x80, 0]);
    assert!(atom.receive_known_body(&mut malformed).is_err());
    assert_eq!(malformed.remaining_bytes(), &[0x80, 0]);
    assert_eq!(atom.item, before);
}
