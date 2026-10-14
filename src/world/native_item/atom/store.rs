use super::NativeItemAtom;
use crate::world::{
    native_atom::NativeAtomKind,
    native_subatoms::{
        NativeCacheNode, NativeNormalSubatoms, NativeRecoveryLookup, NativeRetiredAtoms,
    },
};
use std::{collections::BTreeMap, io};

const MAX_OBJECTS: usize = 20_000;

/// Concrete modeled Item state by stable object handle, never by wire ID.
/// Other atom classes require their own verified constructors/state stores.
#[derive(Clone, Debug, Default)]
pub struct NativeItemObjects {
    objects: BTreeMap<u32, NativeItemAtom>,
}
impl NativeItemObjects {
    pub fn get(&self, handle: u32) -> Option<&NativeItemAtom> {
        self.objects.get(&handle)
    }
    pub fn get_mut(&mut self, handle: u32) -> Option<&mut NativeItemAtom> {
        self.objects.get_mut(&handle)
    }
    pub fn len(&self) -> usize {
        self.objects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Construct confirmed Item fields and admit its cache node. No wire-map,
    /// signals or owner addition. The caller supplies the stable handle.
    pub fn construct_candidate(
        &mut self,
        kind: NativeAtomKind,
        handle: u32,
        graph: &mut NativeNormalSubatoms,
    ) -> io::Result<()> {
        if kind != NativeAtomKind::Item {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native class constructor is not implemented in Item storage",
            ));
        }
        if self.objects.contains_key(&handle) || self.objects.len() >= MAX_OBJECTS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "duplicate or over-budget native Item handle",
            ));
        }
        let item = NativeItemAtom::from_default_item_constructor();
        graph.insert_candidate(
            handle,
            NativeCacheNode {
                kind,
                field_1c: item.base.field_1c,
                position: item.base.position.clone(),
            },
        )?;
        self.objects.insert(handle, item);
        Ok(())
    }
    /// Discard the temporary's graph and modeled Item fields, not shared wire-ID
    /// indexes. Guards run before either store changes. Native Qt locks and
    /// unmodeled C++ destructor effects are not claimed by dropping Rust fields.
    pub fn discard_candidate(
        &mut self,
        handle: u32,
        graph: &mut NativeNormalSubatoms,
        lookup: &NativeRecoveryLookup,
        retired: &NativeRetiredAtoms,
    ) -> io::Result<()> {
        let item = self.objects.get(&handle).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "unknown native Item handle")
        })?;
        graph.discard_candidate(handle, item.base.owner_attached, lookup, retired)?;
        self.objects.remove(&handle);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::native_prefix::NativeIdentityPrefix;
    #[test]
    fn per_handle_items_do_not_alias_shared_wire_id_and_disposal_preserves_retained_object() {
        let mut graph = NativeNormalSubatoms::new([], []).unwrap();
        let mut store = NativeItemObjects::default();
        for handle in [10, 90] {
            store
                .construct_candidate(NativeAtomKind::Item, handle, &mut graph)
                .unwrap();
            store
                .get_mut(handle)
                .unwrap()
                .base
                .apply_received_identity(NativeIdentityPrefix {
                    id: 7,
                    timestamp_ms: 123,
                });
        }
        store.get_mut(10).unwrap().item.ownership_id = Some(99);
        assert_eq!(store.get(90).unwrap().item.ownership_id, Some(0));
        let lookup = NativeRecoveryLookup::new([(7, 10)], []).unwrap();
        let retired = NativeRetiredAtoms::default();
        assert!(
            store
                .discard_candidate(10, &mut graph, &lookup, &retired)
                .is_err()
        );
        store
            .discard_candidate(90, &mut graph, &lookup, &retired)
            .unwrap();
        assert_eq!(store.len(), 1);
        assert_eq!(store.get(10).unwrap().item.ownership_id, Some(99));
        assert!(graph.node(10).is_some());
        assert!(graph.node(90).is_none());
        assert_eq!(lookup.find_atom(7), Some(10));
    }
    #[test]
    fn unsupported_and_duplicate_construction_leave_both_stores_unchanged() {
        let mut graph = NativeNormalSubatoms::new([], []).unwrap();
        let mut store = NativeItemObjects::default();
        assert_eq!(
            store
                .construct_candidate(NativeAtomKind::Person, 90, &mut graph)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert!(store.is_empty());
        assert!(graph.node(90).is_none());
        store
            .construct_candidate(NativeAtomKind::Item, 90, &mut graph)
            .unwrap();
        assert!(
            store
                .construct_candidate(NativeAtomKind::Item, 90, &mut graph)
                .is_err()
        );
        assert_eq!(store.len(), 1);
        assert_eq!(graph.node(90).unwrap().field_1c, 5);
    }
}
