use super::*;
use crate::protocol::QtReader;

fn read(bytes: &[u8]) -> NativeItemBody {
    NativeItemBody::read(&mut QtReader::new(bytes))
        .unwrap()
        .unwrap()
}

#[test]
fn omission_preserves_outer_item_but_replacement_resets_inner_common_fields() {
    let mut state = NativeItemState::default();
    assert_eq!(state.ownership_id, None);
    assert_eq!(state.field_39c, None);
    // State item: explicit byte14/name, unconditional byte20, and state byte.
    state.apply(read(&[0x21, 6, 12, 0, 1, 0xff, 0, 0, 0, 2, 0, 65, 7, 0x80]));
    let original = state.item.clone().unwrap();
    assert_eq!(original.common.effective_name(), Some("A"));
    assert_eq!(original.common.field_14, Some(255));
    assert_eq!(state.field_39c, Some(true));
    state.apply(read(&[0]));
    assert_eq!(state.item, Some(original));
    assert_eq!(state.field_39c, Some(false)); // replaced even without item field
    state.apply(read(&[1, 1, 0, 0, 1, 0]));
    let replaced = state.item.unwrap();
    assert_eq!(replaced.extension.type_id(), 1);
    assert_eq!(replaced.common.effective_name(), Some(""));
    assert_eq!(replaced.common.field_14, None); // catalog-dependent, not old 255
    assert_eq!(replaced.common.field_20(), 1); // wire zero normalization
}

#[test]
fn ownership_sentinel_preserves_strings_and_explicit_null_is_known() {
    let mut state = NativeItemState::default();
    let mut wire = vec![0xc0]; // 0x80 wins over 0x40
    wire.extend(77_u32.to_be_bytes());
    wire.extend(u32::MAX.to_be_bytes());
    wire.extend(0_u32.to_be_bytes());
    state.apply(read(&wire));
    assert_eq!(state.ownership_id, Some(77));
    assert_eq!(state.field_3e0, Some(None));
    assert_eq!(state.field_3e8, Some(Some("".into())));
    state.apply(read(&[0x40]));
    assert_eq!(state.ownership_id, Some(1));
    assert_eq!(state.field_3e0, Some(None));
    assert_eq!(state.field_3e8, Some(Some("".into())));
    state.apply(read(&[0]));
    assert_eq!(state.ownership_id, Some(1));
}

#[test]
fn opaque_updates_invalidate_knowledge_and_sparse_updates_do_not_restore_it() {
    let mut state = NativeItemState::default();
    state.apply(read(&[1, 1, 0, 0, 1, 1]));
    state.invalidate_opaque();
    state.invalidate_opaque();
    assert_eq!(state.item, None);
    assert_eq!(state.field_39c, None);
    assert_eq!(state.opaque_updates, 2);
    state.apply(read(&[0x60]));
    assert_eq!(state.item, None);
    assert_eq!(state.ownership_id, Some(1));
    assert_eq!(state.field_3e0, None);
    assert_eq!(state.field_39c, Some(true));
    assert_eq!(state.opaque_updates, 2);
}
