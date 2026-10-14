use super::*;
use crate::{protocol::QtReader, world::native_item::NativeItemBody};

fn update(state: &mut NativeItemState, bytes: &[u8]) {
    state.apply(
        NativeItemBody::read(&mut QtReader::new(bytes))
            .unwrap()
            .unwrap(),
    );
}

#[test]
fn constructor_defaults_are_explicit_not_unobserved_replay_defaults() {
    let initial = NativeItemInitialization::from_default_item_constructor();
    assert_eq!(initial.base.field_1c, 5);
    assert_eq!(initial.base.wire_id, 0);
    assert!(!initial.base.owner_attached);
    assert_eq!(initial.state.value_present, Some(false));
    assert_eq!(initial.state.item, None);
    assert_eq!(initial.state.ownership_id, Some(0));
    assert_eq!(initial.state.field_3e0, Some(None));
    assert_eq!(initial.state.field_3e8, Some(None));
    assert_eq!(initial.state.field_39c, Some(false));
    let observed = NativeItemState::default();
    assert_eq!(observed.value_present, None);
    assert_eq!(observed.ownership_id, None);
    assert_eq!(observed.field_3e0, None);
}

#[test]
fn sparse_updates_preserve_ctor_knowledge_and_received_value_establishes_presence() {
    let mut initial = NativeItemInitialization::from_default_item_constructor();
    update(&mut initial.state, &[0]);
    assert_eq!(initial.state.value_present, Some(false));
    assert_eq!(initial.state.ownership_id, Some(0));
    update(&mut initial.state, &[0x40]);
    assert_eq!(initial.state.ownership_id, Some(1));
    assert_eq!(initial.state.field_3e0, Some(None));
    assert_eq!(initial.state.field_3e8, Some(None));
    update(&mut initial.state, &[1, 1, 0, 0, 1, 1]);
    assert_eq!(initial.state.value_present, Some(true));
    assert!(initial.state.item.is_some());
    initial.state.invalidate_opaque();
    assert_eq!(initial.state.value_present, None);
    assert_eq!(initial.state.ownership_id, None);
    update(&mut initial.state, &[0]);
    assert_eq!(initial.state.value_present, None);
    assert_eq!(initial.base.field_1c, 5);
}
