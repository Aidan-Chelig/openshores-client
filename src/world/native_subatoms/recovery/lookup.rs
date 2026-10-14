use super::super::{MAX_NODES, invalid};
use std::{collections::BTreeMap, io};

/// Explicit native primary/secondary ID-to-node resolution. Values are stable
/// handles in NativeNormalSubatoms, not invented owner edges or class defaults.
#[derive(Clone, Debug)]
pub struct NativeRecoveryLookup {
    primary: BTreeMap<u32, u32>,
    secondary: BTreeMap<u32, u32>,
}
fn collect(entries: impl IntoIterator<Item = (u32, u32)>) -> io::Result<BTreeMap<u32, u32>> {
    let mut map = BTreeMap::new();
    for (id, handle) in entries {
        if map.len() >= MAX_NODES {
            return Err(invalid("native recovery lookup exceeds budget"));
        }
        if map.insert(id, handle).is_some() {
            return Err(invalid("duplicate native recovery lookup ID"));
        }
    }
    Ok(map)
}
impl NativeRecoveryLookup {
    pub fn new(
        primary: impl IntoIterator<Item = (u32, u32)>,
        secondary: impl IntoIterator<Item = (u32, u32)>,
    ) -> io::Result<Self> {
        Ok(Self {
            primary: collect(primary)?,
            secondary: collect(secondary)?,
        })
    }
    pub(super) fn resolve(&self, id: u32) -> Option<u32> {
        self.primary
            .get(&id)
            .or_else(|| self.secondary.get(&id))
            .copied()
    }
}
