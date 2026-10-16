//! Explicit reference-server player DNA adapter; not native packet dispatch.
use super::{ClientModel, Phase};
use crate::world::{AtomKind, native_dna::NativeDhDna};
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalPlayerDna {
    /// Receive ownership, separate from the eventual geometry cache key.
    pub entity_id: u32,
    /// Retained exactly as received; never replaced with offline fixture DNA.
    pub received: [u8; 24],
    /// Native model-storage normalization, separate from received bytes.
    pub model: NativeDhDna,
}

impl ClientModel {
    /// None means no active source-profile player or DNA has not arrived yet.
    /// Explicit null/invalid DNA is an error, not a successful placeholder.
    /// This intentionally does not infer native armor fields from source gear.
    pub fn local_player_dna(&self) -> io::Result<Option<LocalPlayerDna>> {
        if self.phase != Phase::Playing {
            return Ok(None);
        }
        let Some(id) = self.avatar else {
            return Ok(None);
        };
        let Some(entity) = self.world.entities.get(&id) else {
            return Ok(None);
        };
        if entity.kind != AtomKind::Person {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "selected player entity is not a person",
            ));
        }
        let Some(person) = &entity.person else {
            return Ok(None);
        };
        let Some(raw) = &person.dna else {
            return Ok(None);
        };
        let raw = raw.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "selected player has null DNA")
        })?;
        let model = NativeDhDna::from_received_bytes(raw)?;
        let received = raw.as_slice().try_into().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "selected player DNA requires 24 bytes",
            )
        })?;
        Ok(Some(LocalPlayerDna {
            entity_id: id,
            received,
            model,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{person::PersonState, world::Entity};

    fn playing(raw: Option<Option<Vec<u8>>>) -> ClientModel {
        let mut model = ClientModel {
            phase: Phase::Playing,
            avatar: Some(42),
            ..Default::default()
        };
        let mut entity = Entity {
            id: 42,
            kind: AtomKind::Person,
            timestamp_ms: 0,
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
        };
        entity.person = Some(PersonState {
            dna: raw,
            ..Default::default()
        });
        model.world.entities.insert(42, entity);
        model
    }

    #[test]
    fn player_dna_waits_without_fabrication_and_rejects_null_or_bad_lengths() {
        assert!(ClientModel::default().local_player_dna().unwrap().is_none());
        assert!(playing(None).local_player_dna().unwrap().is_none());
        assert!(playing(Some(None)).local_player_dna().is_err());
        for n in [0, 1, 23, 25, 4096] {
            assert!(playing(Some(Some(vec![0; n]))).local_player_dna().is_err());
        }
    }

    #[test]
    fn actual_received_dna_is_preserved_and_only_model_storage_is_normalized() {
        let raw = vec![0xff; 24];
        let mut model = playing(Some(Some(raw.clone())));
        let dna = model.local_player_dna().unwrap().unwrap();
        assert_eq!(dna.entity_id, 42);
        assert_eq!(dna.received, [0xff; 24]);
        assert_eq!(dna.model.0[5], 0x7f);
        assert_eq!(dna.model.0[23], 0);
        assert_eq!(
            model.world.entities[&42].person.as_ref().unwrap().dna,
            Some(Some(raw))
        );
        // Supplied server emits this byte from person_pose, not an anatomy tag.
        model
            .world
            .entities
            .get_mut(&42)
            .unwrap()
            .person
            .as_mut()
            .unwrap()
            .dna_tag = Some(99);
        assert_eq!(model.local_player_dna().unwrap().unwrap(), dna);
        model.phase = Phase::Disconnected;
        assert!(model.local_player_dna().unwrap().is_none());
    }

    #[test]
    fn selection_and_protocol_phase_determine_which_received_person_is_used() {
        let mut model = playing(Some(Some(vec![1; 24])));
        let mut other = model.world.entities[&42].clone();
        other.id = 43;
        other.person.as_mut().unwrap().dna = Some(Some(vec![2; 24]));
        model.world.entities.insert(43, other);
        model.avatar = Some(43);
        assert_eq!(model.local_player_dna().unwrap().unwrap().received, [2; 24]);
        model.phase = Phase::NativePreview;
        assert!(model.local_player_dna().unwrap().is_none());
        model.phase = Phase::Playing;
        model.world.entities.get_mut(&43).unwrap().kind = AtomKind::Item;
        assert!(model.local_player_dna().is_err());
        model.avatar = Some(99);
        assert!(model.local_player_dna().unwrap().is_none());
    }
}
