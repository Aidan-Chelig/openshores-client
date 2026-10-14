use super::{NativeAtomKind, NativeAtomPrefix, item_body};
use crate::world::native_item::NativeItemBody;
use std::io;

impl NativeAtomPrefix<'_> {
    /// Decode the known Item receiver using the actual selected object's kind.
    /// ReadPacket dispatches the retained object's vtable, not its temporary
    /// candidate's class. The caller must obtain this kind from the selected
    /// object; this method neither resolves identity nor creates/replaces atoms.
    /// Unknown base layers or other retained classes remain opaque.
    pub fn retained_item_body(
        &self,
        retained_kind: NativeAtomKind,
    ) -> io::Result<Option<NativeItemBody>> {
        item_body(retained_kind, self.base.tail_kind, self.remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::native_item::{NativeItemInitialization, OwnershipUpdate};

    #[test]
    fn retained_class_controls_body_and_sparse_item_state() {
        for opcode in [0x05, 0x16, 0x24] {
            let mut bytes = vec![opcode];
            bytes.extend(7_u32.to_be_bytes());
            bytes.extend((-123_i64).to_be_bytes());
            bytes.extend([0, 0x60, 0xde, 0xad]);
            let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
            let body = prefix
                .retained_item_body(NativeAtomKind::Item)
                .unwrap()
                .unwrap();
            assert_eq!(body.consumed_bytes, 1);
            assert_eq!(body.ownership, OwnershipUpdate::Sentinel);
            assert!(body.field_39c);
            assert_eq!(&prefix.remainder[body.consumed_bytes..], &[0xde, 0xad]);
            let mut item = NativeItemInitialization::from_default_item_constructor();
            item.state.apply(body);
            assert_eq!(item.state.ownership_id, Some(1));
            assert_eq!(
                prefix.retained_item_body(NativeAtomKind::Person).unwrap(),
                None
            );
            assert_eq!(prefix.base.timestamp_ms, -123);
        }
    }

    #[test]
    fn unknown_base_layers_prevent_retained_body_decode() {
        for flags in [0x10, 0x40, 0x50] {
            let mut bytes = vec![0x05];
            bytes.extend(7_u32.to_be_bytes());
            bytes.extend(0_i64.to_be_bytes());
            bytes.push(flags);
            let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
            assert_eq!(
                prefix.retained_item_body(NativeAtomKind::Item).unwrap(),
                None
            );
        }
    }
}
