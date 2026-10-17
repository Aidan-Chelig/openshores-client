//! Source-profile sparse state and scene dispatch, not native owner lifecycle.
use super::{AtomKind, AtomUpdate, Transform, WorldIdentity, decode_atom, invalid};
use crate::{
    geology::Geology,
    item::ItemState,
    person::PersonState,
    scene::{self, SceneTime},
    terrain::GlobeSurface,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub id: u32,
    pub kind: AtomKind,
    pub timestamp_ms: i64,
    pub parent: Option<u32>,
    pub created_ms: Option<i64>,
    pub modified_ms: Option<i64>,
    pub transform: Option<Transform>,
    pub name: Option<String>,
    pub world_identity: Option<WorldIdentity>,
    pub globe_surface: Option<GlobeSurface>,
    pub geology: Option<Geology>,
    pub person: Option<PersonState>,
    pub item: Option<ItemState>,
    /// Latest update body only; full packet history belongs to the capture.
    pub last_body: Vec<u8>,
    pub last_base_flags: u8,
}

#[derive(Default)]
pub struct WorldState {
    pub commodities: crate::commodity::CommodityCatalog,
    pub entities: BTreeMap<u32, Entity>,
    /// Manifest visibility is separate from cached atom identity.
    pub visible_ids: BTreeSet<u32>,
    pub time_low: Option<u32>,
    pub active_unit: Option<u32>,
    pub packet_counts: BTreeMap<u8, usize>,
    pub opaque_packet_counts: BTreeMap<u8, usize>,
}

impl WorldState {
    /// Merge in receive order. Omitted fields preserve earlier state.
    /// No original-client clock ordering or deletion semantics are assumed.
    pub fn apply(&mut self, update: AtomUpdate) -> io::Result<()> {
        if self
            .entities
            .get(&update.id)
            .is_some_and(|entity| entity.kind != update.kind)
        {
            return Err(invalid("atom ID changed type"));
        }
        // Parents may arrive later; reject known cycles without partial mutation.
        if let Some(mut parent) = update.parent {
            let mut seen = BTreeSet::from([update.id]);
            while parent != 0 {
                if !seen.insert(parent) {
                    return Err(invalid("cycle in atom parent hierarchy"));
                }
                match self.entities.get(&parent).and_then(|entity| entity.parent) {
                    Some(next) => parent = next,
                    None => break,
                }
            }
        }
        let entity = self.entities.entry(update.id).or_insert_with(|| Entity {
            id: update.id,
            kind: update.kind,
            timestamp_ms: update.timestamp_ms,
            parent: None,
            created_ms: None,
            modified_ms: None,
            transform: None,
            name: None,
            world_identity: None,
            globe_surface: None,
            geology: None,
            person: None,
            item: None,
            last_body: Vec::new(),
            last_base_flags: 0,
        });
        entity.timestamp_ms = update.timestamp_ms;
        if let Some(parent) = update.parent {
            entity.parent = Some(parent);
        }
        if let Some(created) = update.created_ms {
            entity.created_ms = Some(created);
        }
        if let Some(modified) = update.modified_ms {
            entity.modified_ms = Some(modified);
        }
        if let Some(transform) = update.transform {
            entity.transform = Some(transform);
        }
        if let Some(name) = update.name {
            entity.name = name;
        }
        if let Some(identity) = update.world_identity {
            entity.world_identity = Some(identity);
        }
        if let Some(surface) = update.globe_surface {
            entity.globe_surface = Some(surface);
        }
        if let Some(geology) = update.geology {
            entity.geology = Some(geology);
        }
        if let Some(person) = update.person {
            entity
                .person
                .get_or_insert_with(PersonState::default)
                .merge(person);
        }
        if let Some(item) = update.item {
            entity
                .item
                .get_or_insert_with(ItemState::default)
                .merge(item);
        }
        entity.last_body = update.body;
        entity.last_base_flags = update.base_flags;
        Ok(())
    }

    pub fn ingest(&mut self, packet: &[u8]) -> io::Result<()> {
        self.ingest_transform(packet).map(|_| ())
    }

    /// Explicit source transform receipt, including identical values.
    /// Sparse omissions and non-atom packets return None.
    pub fn ingest_transform(&mut self, packet: &[u8]) -> io::Result<Option<(u32, Transform)>> {
        let mut transformed = None;
        let opcode = *packet
            .first()
            .ok_or_else(|| invalid("empty scene packet"))?;
        if let Some(update) = decode_atom(packet)? {
            transformed = update.transform.map(|value| (update.id, value));
            self.apply(update)?;
        } else {
            match opcode {
                0x2b => self.commodities.ingest(packet)?,
                0x18 => match scene::decode_scene_time(packet)? {
                    SceneTime::Manifest { time_low, atom_ids } => {
                        self.time_low = Some(time_low);
                        self.visible_ids = atom_ids.into_iter().collect();
                    }
                    SceneTime::Heartbeat { time_low } => self.time_low = Some(time_low),
                },
                0x2a => self.active_unit = Some(scene::decode_active_unit(packet)?),
                0x38 => {
                    scene::decode_init_succeeded(packet)?;
                }
                _ => {
                    *self.opaque_packet_counts.entry(opcode).or_default() += 1;
                }
            }
        }
        *self.packet_counts.entry(opcode).or_default() += 1;
        Ok(transformed)
    }

    pub fn missing_manifest_ids(&self) -> Vec<u32> {
        self.visible_ids
            .iter()
            .filter(|id| !self.entities.contains_key(id))
            .copied()
            .collect()
    }

    pub fn unresolved_parents(&self) -> BTreeSet<u32> {
        self.entities
            .values()
            .filter_map(|entity| entity.parent)
            .filter(|id| *id != 0 && !self.entities.contains_key(id))
            .collect()
    }

    /// Returns raw parent identities, not composed transforms or world positions.
    pub fn ancestry(&self, id: u32) -> io::Result<Vec<u32>> {
        let mut chain = Vec::new();
        let mut current = id;
        let mut seen = BTreeSet::new();
        while current != 0 {
            if !seen.insert(current) {
                return Err(invalid("cycle in atom parent hierarchy"));
            }
            let entity = self.entities.get(&current).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("unknown ancestor {current:#010x}"),
                )
            })?;
            chain.push(current);
            current = entity.parent.unwrap_or(0);
        }
        Ok(chain)
    }
}
