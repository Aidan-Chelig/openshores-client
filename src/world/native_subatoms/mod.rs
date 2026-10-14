//! Explicit native normal-subatom context; not complete AuEther orchestration.
mod arrival;
mod invalidation;
mod recovery;
mod removal;
mod retirement;
pub use arrival::NativeFreshOwnerEstablished;
pub use removal::NativeAtomRemoval;
pub use retirement::NativeRetiredAtoms;
mod registration;
mod reparent;
mod runaway;
use super::native_atom::NativeAtomKind;
use crate::atom_position::NativePositionState;
pub use invalidation::NativeHierarchyPositionChange;
pub use recovery::{NativeRecoveryLookup, NativeRunawayRecovery};
pub use registration::{ReparentRegistrationContext, RunawayRegistrationStores};
pub use reparent::NativeReparentResult;
pub use runaway::NativeRunawayReparentResult;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

const MAX_NODES: usize = 20_000;
const MAX_LINKS: usize = 128_000;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeCacheNode {
    pub kind: NativeAtomKind,
    /// Native offset 0x1c. Meaning/wire source unestablished; do not invent a
    /// default from base flags. AppendNormal requires this raw field to be zero.
    pub field_1c: i32,
    pub position: NativePositionState,
}

#[derive(Clone, Debug)]
pub struct NativeChildList {
    /// Native ListNormalSubatoms first requires a parent-lock map entry.
    /// Availability is explicit, not inferred from parent node presence.
    pub available: bool,
    pub children: Vec<u32>,
}

#[derive(Clone, Debug)]
struct ChildSet {
    available: bool,
    children: BTreeSet<u32>,
}

#[derive(Clone, Debug)]
pub struct NativeNormalSubatoms {
    nodes: BTreeMap<u32, NativeCacheNode>,
    lists: BTreeMap<u32, ChildSet>,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

impl NativeNormalSubatoms {
    /// Explicit owner context, not edges guessed from packed parent fields.
    /// Rust safety budgets/duplicate and missing-reference checks are not native
    /// registration policy. No external state changes on construction failure.
    pub fn new(
        nodes: impl IntoIterator<Item = (u32, NativeCacheNode)>,
        lists: impl IntoIterator<Item = (u32, NativeChildList)>,
    ) -> io::Result<Self> {
        let mut result = Self {
            nodes: BTreeMap::new(),
            lists: BTreeMap::new(),
        };
        for (id, node) in nodes {
            if result.nodes.len() >= MAX_NODES {
                return Err(invalid("native subatoms exceed node budget"));
            }
            if result.nodes.insert(id, node).is_some() {
                return Err(invalid("duplicate native subatom ID"));
            }
        }
        let mut links = 0usize;
        for (parent, list) in lists {
            if result.lists.len() >= MAX_NODES || list.children.len() > MAX_LINKS - links {
                return Err(invalid("native subatoms exceed list/link budget"));
            }
            links += list.children.len();
            if list
                .children
                .iter()
                .any(|id| !result.nodes.contains_key(id))
            {
                return Err(invalid("native subatom list references a missing node"));
            }
            let value = ChildSet {
                available: list.available,
                children: list.children.into_iter().collect(),
            };
            if result.lists.insert(parent, value).is_some() {
                return Err(invalid("duplicate native child list"));
            }
        }
        Ok(result)
    }

    pub fn node(&self, id: u32) -> Option<&NativeCacheNode> {
        self.nodes.get(&id)
    }
}

#[cfg(test)]
mod tests;
