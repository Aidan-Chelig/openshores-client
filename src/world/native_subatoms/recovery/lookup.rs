use super::super::{MAX_NODES, invalid};
use crate::world::native_atom::NativeAtomKind;
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
        insert(&mut map, id, handle)?;
    }
    Ok(map)
}
fn insert(map: &mut BTreeMap<u32, u32>, id: u32, handle: u32) -> io::Result<()> {
    if map.len() >= MAX_NODES {
        return Err(invalid("native recovery lookup exceeds budget"));
    }
    if map.insert(id, handle).is_some() {
        return Err(invalid("duplicate native recovery lookup ID"));
    }
    Ok(())
}
impl NativeRecoveryLookup {
    pub(in crate::world::native_subatoms) fn insert_routed(
        &mut self,
        id: u32,
        handle: u32,
        kind: NativeAtomKind,
    ) -> io::Result<()> {
        let map = if kind.in_star_field() {
            &mut self.secondary
        } else {
            &mut self.primary
        };
        if map.len() >= MAX_NODES && !map.contains_key(&id) {
            return Err(invalid("native routed atom map exceeds budget"));
        }
        map.insert(id, handle);
        Ok(())
    }
    pub(in crate::world::native_subatoms) fn remove_routed(
        &mut self,
        id: u32,
        kind: NativeAtomKind,
    ) -> Option<u32> {
        if kind.in_star_field() {
            self.secondary.remove(&id)
        } else {
            self.primary.remove(&id)
        }
    }
    /// Build explicit recovery maps from established node identities/kinds,
    /// using native InStarField routing. This is not AddAtomLocked's lifecycle:
    /// replacement/removal, owner creation and callbacks remain separate.
    pub fn from_routed_nodes(
        entries: impl IntoIterator<Item = (u32, u32, NativeAtomKind)>,
    ) -> io::Result<Self> {
        let mut result = Self {
            primary: BTreeMap::new(),
            secondary: BTreeMap::new(),
        };
        for (id, handle, kind) in entries {
            let map = if kind.in_star_field() {
                &mut result.secondary
            } else {
                &mut result.primary
            };
            insert(map, id, handle)?;
        }
        Ok(result)
    }
    pub fn new(
        primary: impl IntoIterator<Item = (u32, u32)>,
        secondary: impl IntoIterator<Item = (u32, u32)>,
    ) -> io::Result<Self> {
        Ok(Self {
            primary: collect(primary)?,
            secondary: collect(secondary)?,
        })
    }
    pub(in crate::world::native_subatoms) fn routed_entry(
        &self,
        id: u32,
        kind: NativeAtomKind,
    ) -> Option<u32> {
        if kind.in_star_field() {
            self.secondary.get(&id)
        } else {
            self.primary.get(&id)
        }
        .copied()
    }
    pub(in crate::world::native_subatoms) fn resolve(&self, id: u32) -> Option<u32> {
        self.primary
            .get(&id)
            .or_else(|| self.secondary.get(&id))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routed_nodes_follow_star_field_not_environment_boundaries() {
        use NativeAtomKind::*;
        let result = NativeRecoveryLookup::from_routed_nodes([
            (1, 10, Item),
            (2, 20, Arena),
            (3, 30, Galaxy),
            (4, 40, Universe),
            (5, 50, Sector),
            (6, 60, SolarSystem),
            (7, 70, Star),
            (8, 80, Designer),
            (1, 11, Universe),
        ])
        .unwrap();
        assert_eq!(
            result.primary.keys().copied().collect::<Vec<_>>(),
            [1, 2, 7, 8]
        );
        assert_eq!(
            result.secondary.keys().copied().collect::<Vec<_>>(),
            [1, 3, 4, 5, 6]
        );
        assert_eq!(result.resolve(1), Some(10));
        assert!(NativeRecoveryLookup::from_routed_nodes([(1, 10, Item), (1, 11, Person)]).is_err());
    }
}
