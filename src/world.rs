//! Scene atom prefixes and sparse state, independent of rendering/UI frameworks.
//! Type-specific trailing bytes remain available without claiming full decoding.
use crate::{
    geology::Geology,
    person::PersonState,
    protocol::{QtReader, read_frame},
    scene::{self, SceneTime},
    terrain::{GlobeSurface, TerrainParameters},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, BufRead},
};

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AtomKind {
    Universe,
    Galaxy,
    Sector,
    SolarSystem,
    Star,
    Globe,
    GasGiant,
    Ring,
    RingSection,
    Person,
    Vehicle,
    Building,
}

impl AtomKind {
    pub fn from_opcode(opcode: u8) -> Option<Self> {
        Some(match opcode {
            0x1b => Self::Universe,
            0x10 => Self::Galaxy,
            0x14 => Self::Sector,
            0x15 => Self::SolarSystem,
            0x17 => Self::Star,
            0x1f => Self::Globe,
            0x1e => Self::GasGiant,
            0x20 => Self::Ring,
            0x40 => Self::RingSection,
            0x12 => Self::Person,
            0x1c => Self::Vehicle,
            0x4c => Self::Building,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub position: [f32; 3],
    /// Raw wire rotations; conversion/units are not established for every kind.
    pub rotation: [f32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldIdentity {
    pub zone: u8,
    pub orbit_au: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AtomUpdate {
    pub id: u32,
    pub kind: AtomKind,
    pub timestamp_ms: i64,
    pub base_flags: u8,
    pub parent: Option<u32>,
    pub created_ms: Option<i64>,
    pub modified_ms: Option<i64>,
    pub transform: Option<Transform>,
    /// Outer None means omitted; Some(None) is an explicit Qt null string.
    pub name: Option<Option<String>>,
    pub world_identity: Option<WorldIdentity>,
    pub globe_surface: Option<GlobeSurface>,
    pub geology: Option<Geology>,
    pub person: Option<PersonState>,
    /// Full type-specific body, including any fields decoded above.
    pub body: Vec<u8>,
}

pub fn decode_atom(bytes: &[u8]) -> io::Result<Option<AtomUpdate>> {
    let mut input = QtReader::new(bytes);
    let Some(kind) = AtomKind::from_opcode(input.read_u8()?) else {
        return Ok(None);
    };
    let id = input.read_u32()?;
    if id == 0 {
        return Err(invalid("scene atom ID must be nonzero"));
    }
    let timestamp_ms = input.read_i64()?;
    let base_flags = input.read_u8()?;
    let parent = if base_flags & 1 != 0 {
        Some(input.read_u32()?)
    } else {
        None
    };
    let created_ms = if base_flags & 2 != 0 {
        Some(input.read_i64()?)
    } else {
        None
    };
    let modified_ms = if base_flags & 4 != 0 {
        Some(input.read_i64()?)
    } else {
        None
    };
    let transform = if base_flags & 8 != 0 {
        let values = (0..6)
            .map(|_| input.read_f32())
            .collect::<io::Result<Vec<_>>>()?;
        if values.iter().any(|value| !value.is_finite()) {
            return Err(invalid("non-finite atom transform"));
        }
        Some(Transform {
            position: values[..3].try_into().unwrap(),
            rotation: values[3..].try_into().unwrap(),
        })
    } else {
        None
    };
    let body = bytes[bytes.len() - input.remaining()..].to_vec();
    let mut name = None;
    let mut world_identity = None;
    let mut globe_surface = None;
    let mut geology = None;
    let mut person = None;
    // Effect payloads and unknown base layers precede type-specific fields.
    // Preserve them as body bytes until their layouts are established.
    if base_flags & 0xd0 == 0 {
        match kind {
            AtomKind::Sector | AtomKind::SolarSystem => name = Some(input.read_string()?),
            AtomKind::Person | AtomKind::Vehicle | AtomKind::Building => {
                let flags = input.read_u8()?;
                if flags & 1 != 0 {
                    name = Some(input.read_string()?);
                }
                if kind == AtomKind::Person {
                    person = Some(PersonState::read_prefix(&mut input, flags)?);
                }
            }
            AtomKind::Globe | AtomKind::GasGiant | AtomKind::RingSection => {
                let flags = input.read_u8()?;
                // Only the source-established WgWorld flag-0/flag-1 prefix is read.
                if flags & !1 == 0 {
                    if flags & 1 != 0 {
                        let zone = input.read_u8()?;
                        let orbit_au = input.read_f64()?;
                        if !orbit_au.is_finite() {
                            return Err(invalid("non-finite world orbit"));
                        }
                        world_identity = Some(WorldIdentity { zone, orbit_au });
                    }
                    name = Some(input.read_string()?);
                    if kind == AtomKind::Globe {
                        let globe_flags = input.read_u8()?;
                        if globe_flags & 1 != 0 {
                            let size = input.read_u8()?;
                            let atmosphere_type = input.read_u8()?;
                            let atmosphere_density = input.read_u8()?;
                            let water = input.read_u8()?;
                            let values = (0..6)
                                .map(|_| input.read_f32())
                                .collect::<io::Result<Vec<_>>>()?;
                            globe_surface = Some(GlobeSurface {
                                size,
                                atmosphere_type,
                                atmosphere_density,
                                water,
                                terrain: TerrainParameters::new(values.try_into().unwrap())?,
                            });
                        }
                        if globe_flags & 8 != 0 {
                            geology = Some(Geology::read_prefix(&mut input)?);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(Some(AtomUpdate {
        id,
        kind,
        timestamp_ms,
        base_flags,
        parent,
        created_ms,
        modified_ms,
        transform,
        name,
        world_identity,
        globe_surface,
        geology,
        person,
        body,
    }))
}

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
    /// Latest update body only; full packet history belongs to the capture.
    pub last_body: Vec<u8>,
    pub last_base_flags: u8,
}

#[derive(Default)]
pub struct WorldState {
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
        entity.last_body = update.body;
        entity.last_base_flags = update.base_flags;
        Ok(())
    }

    pub fn ingest(&mut self, packet: &[u8]) -> io::Result<()> {
        let opcode = *packet
            .first()
            .ok_or_else(|| invalid("empty scene packet"))?;
        if let Some(update) = decode_atom(packet)? {
            self.apply(update)?;
        } else {
            match opcode {
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
        Ok(())
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

/// Replay scene-only captures with the same bounds as the local network probe.
pub fn read_capture(reader: &mut impl BufRead) -> io::Result<WorldState> {
    let mut world = WorldState::default();
    let mut count = 0;
    let mut bytes = 0usize;
    while !reader.fill_buf()?.is_empty() {
        count += 1;
        if count > 20_000 {
            return Err(invalid("capture exceeds packet budget"));
        }
        let packet = read_frame(reader, 16 * 1024 * 1024)?;
        bytes += packet.len();
        if bytes > 128 * 1024 * 1024 {
            return Err(invalid("capture exceeds byte budget"));
        }
        world.ingest(&packet)?;
    }
    Ok(world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::write_frame;

    fn hex(text: &str) -> Vec<u8> {
        text.trim()
            .as_bytes()
            .chunks_exact(2)
            .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
            .collect()
    }

    #[test]
    fn geology_updates_are_sparse_and_explicit_empty_clears() {
        let mut bytes = hex(include_str!("../tests/fixtures/atom-globe.hex"));
        // Existing fixture ends after class terrain: add flag-8 and a mesa.
        let flag_index = bytes.len() - 29;
        bytes[flag_index] |= 8;
        bytes.extend([1, 3]);
        for value in [2.0f32, 1.0, 3.0, 4.0] {
            bytes.extend(value.to_be_bytes());
        }
        let mut world = WorldState::default();
        world.ingest(&bytes).unwrap();
        let geology = world.entities[&7].geology.clone().unwrap();
        assert_eq!(geology.features[0].latitude, 1.0);
        world
            .ingest(&hex(include_str!(
                "../tests/fixtures/atom-globe-rename.hex"
            )))
            .unwrap();
        assert_eq!(world.entities[&7].geology.as_ref(), Some(&geology));
        bytes.truncate(bytes.len() - 18);
        bytes.push(0);
        world.ingest(&bytes).unwrap();
        assert!(
            world.entities[&7]
                .geology
                .as_ref()
                .unwrap()
                .features
                .is_empty()
        );
    }

    #[test]
    fn globe_rename_preserves_surface_and_rejects_truncated_parameters() {
        let bytes = hex(include_str!("../tests/fixtures/atom-globe.hex"));
        let mut world = WorldState::default();
        world.ingest(&bytes).unwrap();
        let surface = world.entities[&7].globe_surface.unwrap();
        assert_eq!(
            (
                surface.size,
                surface.atmosphere_type,
                surface.atmosphere_density,
                surface.water
            ),
            (8, 0, 60, 40)
        );
        assert_eq!(
            world.entities[&7].world_identity.as_ref().unwrap().orbit_au,
            1.0
        );
        assert!(
            (surface.terrain.altitude_msl(8, 0.0, 0.0).unwrap() - 216.0207753854408).abs() < 1e-8
        );
        world
            .ingest(&hex(include_str!(
                "../tests/fixtures/atom-globe-rename.hex"
            )))
            .unwrap();
        assert_eq!(world.entities[&7].name.as_deref(), Some("Renamed"));
        assert_eq!(world.entities[&7].globe_surface, Some(surface));
        for end in 0..bytes.len() {
            assert!(decode_atom(&bytes[..end]).is_err());
        }
    }

    #[test]
    fn python_sector_fixture_and_sparse_rename() {
        let full = hex(include_str!("../tests/fixtures/atom-sector.hex"));
        let mut world = WorldState::default();
        world.ingest(&full).unwrap();
        let original = world.entities[&42].clone();
        assert_eq!(original.parent, Some(117));
        assert_eq!(original.name.as_deref(), Some("Sector 🚀"));
        assert_eq!(original.transform.unwrap().position, [1.25, -2.5, 3.0]);
        world
            .ingest(&hex(include_str!(
                "../tests/fixtures/atom-sector-rename.hex"
            )))
            .unwrap();
        assert_eq!(world.entities[&42].name.as_deref(), Some("Renamed"));
        assert_eq!(world.entities[&42].transform, original.transform);
        assert_eq!(world.entities[&42].parent, original.parent);
        assert_eq!(world.entities[&42].timestamp_ms, 2000);
        for end in 0..full.len() {
            assert!(decode_atom(&full[..end]).is_err());
        }
    }

    #[test]
    fn cycles_and_type_conflicts_leave_state_unchanged() {
        let mut world = WorldState::default();
        let base = decode_atom(&hex(include_str!("../tests/fixtures/atom-sector.hex")))
            .unwrap()
            .unwrap();
        world.apply(base.clone()).unwrap();
        let original = world.entities.clone();
        let mut cycle = base.clone();
        cycle.parent = Some(42);
        assert!(world.apply(cycle).is_err());
        let mut parent = base.clone();
        parent.id = 117;
        parent.parent = Some(42);
        assert!(world.apply(parent).is_err());
        let mut conflict = base;
        conflict.kind = AtomKind::Person;
        assert!(world.apply(conflict).is_err());
        assert_eq!(world.entities, original);
    }

    #[test]
    fn replay_tracks_manifest_visibility_and_rejects_truncation() {
        let atom = hex(include_str!("../tests/fixtures/atom-sector.hex"));
        let mut capture = Vec::new();
        write_frame(&mut capture, &atom).unwrap();
        write_frame(&mut capture, &hex("18000000010100020000002a00000063")).unwrap();
        write_frame(&mut capture, &[0xfe, 1, 2]).unwrap();
        let mut world = read_capture(&mut capture.as_slice()).unwrap();
        assert_eq!(world.missing_manifest_ids(), [99]);
        assert_eq!(world.unresolved_parents(), BTreeSet::from([117]));
        assert_eq!(world.opaque_packet_counts[&0xfe], 1);
        world.ingest(&hex("1800000002010000")).unwrap();
        assert!(world.visible_ids.is_empty());
        assert_eq!(world.entities.len(), 1);
        assert!(read_capture(&mut [0xc0].as_slice()).is_err());
        assert!(read_capture(&mut [2, 1].as_slice()).is_err());
    }
}
