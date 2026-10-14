//! Patched DtClient::ReadPacket atom dispatch, distinct from reference opcodes.
mod construction;
use super::native_prefix::{NativeBasePrefix, TailKind};
use crate::{
    atom_position::{FloatEncoding, NativePositionState, PositionChange},
    protocol::QtReader,
};
pub use construction::NativeBaseAtomInitialization;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeAtomKind {
    Animal,
    Arena,
    Asteroid,
    Person,
    Building,
    Designer,
    DesignPlane,
    Effect,
    Galaxy,
    Item,
    Missile,
    Npc,
    Spacecraft,
    Sector,
    SolarSystem,
    Star,
    Teleporter,
    Universe,
    Vehicle,
    GasGiant,
    Globe,
    Ring,
    RingSection,
}

impl NativeAtomKind {
    /// Native InStarField slot 0x618. Routes AddAtomLocked to the secondary
    /// star-field map; distinct from environment and instance-universe gates.
    pub fn in_star_field(self) -> bool {
        matches!(
            self,
            Self::Galaxy | Self::Sector | Self::SolarSystem | Self::Universe
        )
    }
    /// Native IsInstanceUniverse predicate, not the Universe opcode or the
    /// distinct CanRenderEnvironment predicate. True only for Arena/Designer.
    pub fn is_instance_universe(self) -> bool {
        matches!(self, Self::Arena | Self::Designer)
    }
    /// Static CanRenderEnvironment vtable bindings in the selected patched image.
    /// This is a parent-boundary predicate, not its EnvTransform implementation.
    pub fn can_render_environment(self) -> bool {
        matches!(
            self,
            Self::Arena | Self::Designer | Self::Sector | Self::SolarSystem
        )
    }

    pub fn from_opcode(opcode: u8) -> Option<Self> {
        Some(match opcode {
            0x02 => Self::Animal,
            0x03 => Self::Arena,
            0x04 => Self::Asteroid,
            0x05 => Self::Person,
            0x06 => Self::Building,
            0x12 => Self::Designer,
            0x13 => Self::DesignPlane,
            0x14 => Self::Effect,
            0x15 => Self::Galaxy,
            0x16 => Self::Item,
            0x17 => Self::Missile,
            0x18 => Self::Npc,
            0x19 => Self::Spacecraft,
            0x1a => Self::Sector,
            0x1c => Self::SolarSystem,
            0x1d => Self::Star,
            0x1e => Self::Teleporter,
            0x20 => Self::Universe,
            0x21 => Self::Vehicle,
            0x23 => Self::GasGiant,
            0x24 => Self::Globe,
            0x27 => Self::Ring,
            0x28 => Self::RingSection,
            _ => return None,
        })
    }
}

/// Known native atom prefix, with its remaining layers retained byte-exactly.
/// No source-profile body parsing, hierarchy mutation or movement interpolation.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeAtomPrefix<'a> {
    pub kind: NativeAtomKind,
    pub base: NativeBasePrefix,
    pub remainder: &'a [u8],
}

impl<'a> NativeAtomPrefix<'a> {
    /// Decode established item subtypes only when no unknown base layer
    /// separates this prefix from the body. Other kinds/subtypes stay opaque.
    pub fn item_body(&self) -> io::Result<Option<super::native_item::NativeItemBody>> {
        item_body(self.kind, self.base.tail_kind, self.remainder)
    }
    /// Caller selects this atom's state by ID. Applies only the decoded fresh
    /// position and its local timer, not opaque body, metadata or hierarchy effects.
    pub fn apply_fresh_position(
        &self,
        state: &mut NativePositionState,
        has_owning_hierarchy: bool,
    ) -> io::Result<Option<PositionChange>> {
        self.base
            .position
            .as_ref()
            .map(|record| state.apply_fresh_wire_position(record, has_owning_hierarchy))
            .transpose()
    }

    /// Input is one unframed native packet payload. Unknown/non-atom opcodes
    /// return None. DtClient's stream version 7 uses type-width float encoding.
    pub fn decode(payload: &'a [u8]) -> io::Result<Option<Self>> {
        let mut input = QtReader::new(payload);
        let Some(kind) = NativeAtomKind::from_opcode(input.read_u8()?) else {
            return Ok(None);
        };
        let base = NativeBasePrefix::read(&mut input, FloatEncoding::TypeWidths)?;
        let remainder = &payload[1 + base.consumed_bytes..];
        Ok(Some(Self {
            kind,
            base,
            remainder,
        }))
    }
}

pub(crate) fn item_body(
    kind: NativeAtomKind,
    tail_kind: TailKind,
    bytes: &[u8],
) -> io::Result<Option<super::native_item::NativeItemBody>> {
    if kind != NativeAtomKind::Item || tail_kind != TailKind::TypeSpecific {
        return Ok(None);
    }
    super::native_item::NativeItemBody::read(&mut QtReader::new(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_person_item_and_globe_do_not_use_reference_dispatch() {
        for (opcode, expected) in [
            (0x05, NativeAtomKind::Person),
            (0x16, NativeAtomKind::Item),
            (0x24, NativeAtomKind::Globe),
        ] {
            let mut bytes = vec![opcode];
            bytes.extend(7_u32.to_be_bytes());
            bytes.extend(123_i64.to_be_bytes());
            bytes.extend([1, 0x82]); // base position; packed double X, no parent
            bytes.extend(12345678.125_f64.to_be_bytes());
            bytes.extend([0xde, 0xad]);
            let prefix = NativeAtomPrefix::decode(&bytes).unwrap().unwrap();
            assert_eq!(prefix.kind, expected);
            assert_eq!(
                prefix.base.position.as_ref().unwrap().position[0],
                12345678.125
            );
            let mut state = NativePositionState::default();
            assert!(matches!(
                prefix.apply_fresh_position(&mut state, true).unwrap(),
                Some(PositionChange::Applied { .. })
            ));
            assert_eq!(state.position.local[3][0], 12345678.125);
            assert_eq!(prefix.remainder, &[0xde, 0xad]);
            for length in 0..bytes.len() - 2 {
                assert!(NativeAtomPrefix::decode(&bytes[..length]).is_err());
            }
        }
        assert_eq!(
            NativeAtomKind::from_opcode(0x12),
            Some(NativeAtomKind::Designer)
        );
        assert!(NativeAtomPrefix::decode(&[0x11]).unwrap().is_none());
        assert!(NativeAtomPrefix::decode(&[0xff]).unwrap().is_none());
    }
}
