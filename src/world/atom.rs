//! Source-server atom profile; distinct from patched native opcode/layouts.
use super::invalid;
use crate::{
    geology::Geology,
    item::ItemState,
    person::PersonState,
    protocol::QtReader,
    terrain::{GlobeSurface, TerrainParameters},
};
use std::io;

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
    Item,
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
            0x11 => Self::Item,
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
impl Transform {
    /// Source-profile f64 position reports are rebroadcast as f32. Compare at
    /// that wire precision so an echo need not destroy sub-f32 local precision.
    /// This is not a sequence acknowledgement or a native transform comparison.
    pub fn matches_report(&self, position: [f64; 3], rotation: [f64; 3]) -> bool {
        self.position == position.map(|v| v as f32) && self.rotation == rotation.map(|v| v as f32)
    }
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
    pub item: Option<ItemState>,
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
    let mut item = None;
    // Effect payloads and unknown base layers precede type-specific fields.
    // Preserve them as body bytes until their layouts are established.
    if base_flags & 0xd0 == 0 {
        match kind {
            AtomKind::Item => item = Some(ItemState::decode_body(&body)?),
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
        item,
        body,
    }))
}
