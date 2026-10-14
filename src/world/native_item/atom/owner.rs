use super::{NativeItemAtom, NativeItemObjects};
use crate::world::native_subatoms::{
    NativeAtomAdd, NativeAtomAddRequest, NativeAtomOwnerContext, NativeAtomOwnerStores,
    NativeNormalSubatoms, ReparentRegistrationContext,
};
use std::io;

struct ItemOwner<'a, O> {
    item: &'a mut NativeItemAtom,
    owner: &'a mut O,
}
impl<O: NativeAtomOwnerContext> ReparentRegistrationContext for ItemOwner<'_, O> {
    type Context<'a>
        = O::Context<'a>
    where
        Self: 'a;
    fn context<'a>(
        &'a mut self,
        graph: &'a mut NativeNormalSubatoms,
        handle: u32,
    ) -> Self::Context<'a> {
        self.owner.context(graph, handle)
    }
}
impl<O: NativeAtomOwnerContext> NativeAtomOwnerContext for ItemOwner<'_, O> {
    fn detach_owner(&mut self, _: u32) {
        unreachable!("fresh Item addition cannot replace an indexed object")
    }
    fn assign_owner(&mut self, graph: &NativeNormalSubatoms, handle: u32) {
        self.item.base.owner_attached = true;
        self.owner.assign_owner(graph, handle);
    }
}
impl NativeItemObjects {
    /// Fresh ReadPacket AddAtom stage only. Existing IDs must use the retained
    /// object path; this adapter never converts packet reuse into replacement.
    /// Concrete owner metadata is assigned inside the real add lifecycle before
    /// FixRunaways. Prior stages survive Rust errors as in the underlying owner.
    pub fn add_candidate_owner(
        &mut self,
        handle: u32,
        graph: &mut NativeNormalSubatoms,
        stores: NativeAtomOwnerStores<'_>,
        owner: &mut impl NativeAtomOwnerContext,
    ) -> io::Result<NativeAtomAdd> {
        let item = self.get_mut(handle).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "unknown native Item handle")
        })?;
        graph.validate_unowned_candidate(
            handle,
            item.base.owner_attached,
            stores.lookup,
            stores.retired,
        )?;
        if stores.lookup.find_atom(item.base.wire_id).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "existing native ID requires retained-object reception",
            ));
        }
        let request = NativeAtomAddRequest {
            wire_id: item.base.wire_id,
            handle,
            scan_all_memberships: false,
        };
        graph.add_atom_owner(request, stores, &mut ItemOwner { item, owner }, || {
            unreachable!("fresh object has no retirement clock")
        })
    }
}
