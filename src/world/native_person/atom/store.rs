use super::NativePersonAtom;
use crate::world::{
    native_atom::NativeAtomKind,
    native_subatoms::{
        NativeCacheNode, NativeNormalSubatoms, NativeRecoveryLookup, NativeRetiredAtoms,
    },
};
use std::{collections::BTreeMap, io};

const MAX_OBJECTS: usize = 20_000;

/// Concrete modeled Person state by stable object handle, never by wire ID.
/// Other atom classes require their own verified constructors/state stores.
#[derive(Clone, Debug, Default)]
pub struct NativePersonObjects {
    objects: BTreeMap<u32, NativePersonAtom>,
}
impl NativePersonObjects {
    pub fn get(&self, handle: u32) -> Option<&NativePersonAtom> {
        self.objects.get(&handle)
    }
    pub fn get_mut(&mut self, handle: u32) -> Option<&mut NativePersonAtom> {
        self.objects.get_mut(&handle)
    }
    pub fn len(&self) -> usize {
        self.objects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Construct confirmed Person fields and admit its cache node. No wire-map,
    /// signals or owner addition. The caller supplies the stable handle and
    /// separately constructs runtime models; this allocates receive storage only.
    pub fn construct_candidate(
        &mut self,
        kind: NativeAtomKind,
        handle: u32,
        graph: &mut NativeNormalSubatoms,
    ) -> io::Result<()> {
        if kind != NativeAtomKind::Person {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native class constructor is not implemented in Person storage",
            ));
        }
        if self.objects.contains_key(&handle) || self.objects.len() >= MAX_OBJECTS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "duplicate or over-budget native Person handle",
            ));
        }
        let person = NativePersonAtom::from_default_person_constructor_subset();
        graph.insert_candidate(
            handle,
            NativeCacheNode {
                kind,
                field_1c: person.base.field_1c,
                position: person.base.position.clone(),
            },
        )?;
        self.objects.insert(handle, person);
        Ok(())
    }
    /// Discard the temporary's graph and modeled Person fields, not shared wire-ID
    /// indexes. Guards run before either store changes. Native Qt locks and
    /// unmodeled C++ destructor effects are not claimed by dropping Rust fields.
    pub fn discard_candidate(
        &mut self,
        handle: u32,
        graph: &mut NativeNormalSubatoms,
        lookup: &NativeRecoveryLookup,
        retired: &NativeRetiredAtoms,
    ) -> io::Result<()> {
        let person = self.objects.get(&handle).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "unknown native Person handle")
        })?;
        graph.discard_candidate(handle, person.base.owner_attached, lookup, retired)?;
        self.objects.remove(&handle);
        Ok(())
    }
}
