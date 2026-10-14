//! Source-established DaPerson prefix. Unestablished layers are not skipped.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PersonState {
    pub empire: Option<i32>,
    pub identity_tag: Option<u8>,
    pub dna_tag: Option<u8>,
    /// Omission differs from Qt null and empty DNA.
    pub dna: Option<Option<Vec<u8>>>,
    pub head_tilt: Option<f32>,
    pub hp: Option<i16>,
    pub poses: Option<[u8; 10]>,
    pub pose_parameter: Option<u32>,
    pub stamina: Option<u8>,
    pub growth: Option<u8>,
    pub hunger_raw: Option<u16>,
    pub agent_bits: Option<u8>,
    pub creature_flags: Option<u8>,
    pub person_flags: Option<u8>,
    pub cursor_raw: Option<[u8; 3]>,
    /// Last gear update batch, not an inferred authoritative equipment snapshot.
    pub gear: Option<crate::inventory::Gear>,
}

impl PersonState {
    pub fn merge(&mut self, update: Self) {
        macro_rules! merge { ($($field:ident),*) => { $(if update.$field.is_some() { self.$field = update.$field; })* }; }
        merge!(
            empire,
            identity_tag,
            dna_tag,
            dna,
            head_tilt,
            hp,
            poses,
            pose_parameter,
            stamina,
            growth,
            hunger_raw,
            agent_bits,
            creature_flags,
            person_flags,
            cursor_raw,
            gear
        );
    }

    /// Called after the common name field. Stops before unknown-layer payloads.
    pub(crate) fn read_prefix(input: &mut QtReader<'_>, common_flags: u8) -> io::Result<Self> {
        let mut state = Self::default();
        if common_flags & 1 != 0 {
            state.empire = Some(input.read_u32()? as i32);
            let tag = input.read_u8()?;
            state.identity_tag = Some(tag);
            if tag != 0 {
                return Ok(state);
            }
        }
        if common_flags & !1 != 0 {
            return Ok(state);
        }
        let flags = input.read_u8()?;
        state.creature_flags = Some(flags);
        if flags & 1 != 0 {
            state.dna_tag = Some(input.read_u8()?);
            let dna = input.read_bytes()?;
            if dna.is_some_and(|bytes| bytes.len() > 4096) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "person DNA exceeds prefix budget",
                ));
            }
            state.dna = Some(dna.map(|bytes| bytes.to_vec()));
        }
        if flags & 2 != 0 {
            let tilt = input.read_f32()?;
            if !tilt.is_finite() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "non-finite head tilt",
                ));
            }
            state.head_tilt = Some(tilt);
        }
        if flags & 4 != 0 {
            state.hp = Some(input.read_u16()? as i16);
        }
        if flags & 8 != 0 {
            let mut poses = [0; 10];
            for pose in &mut poses {
                *pose = input.read_u8()?;
            }
            state.poses = Some(poses);
            state.pose_parameter = Some(input.read_u32()?);
        }
        if flags & !0x2f != 0 {
            return Ok(state);
        }
        state.stamina = Some(input.read_u8()?);
        state.growth = Some(input.read_u8()?);
        let flags = input.read_u8()?;
        state.person_flags = Some(flags);
        // Citizen/other earlier layers have variable layouts, so don't seek
        // hunger or agent data beyond them. Bootstrap still yields creature state.
        if flags & 3 != 0 {
            return Ok(state);
        }
        if flags & 4 != 0 {
            state.hunger_raw = Some(input.read_u16()?);
        }
        if flags & 8 != 0 {
            state.agent_bits = Some(input.read_u8()?);
        }
        if flags & 0x10 != 0 {
            return Ok(state);
        }
        if flags & 0x20 != 0 {
            state.cursor_raw = Some([input.read_u8()?, input.read_u8()?, input.read_u8()?]);
        }
        if flags & 0x40 != 0 {
            let (gear, consumed) = crate::inventory::Gear::read_prefix(input.remaining_bytes())?;
            input.skip(consumed)?;
            state.gear = Some(gear);
        }
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use crate::world::WorldState;

    fn packet(body: &[u8]) -> Vec<u8> {
        let mut out = vec![0x12];
        out.extend(42u32.to_be_bytes());
        out.extend(1000i64.to_be_bytes());
        out.push(0);
        out.extend(body);
        out
    }

    #[test]
    fn gear_batches_and_cursor_are_explicit_and_atomic() {
        let mut world = WorldState::default();
        let body = [
            0, 0, 90, 0, 0x68, 5, 9, 2, 1, 2, 0x15, 0, 0x25, 1, 4, 0, 130, 5, 61,
        ];
        world.ingest(&packet(&body)).unwrap();
        let before = world.entities[&42].clone();
        let state = before.person.as_ref().unwrap();
        assert_eq!(state.cursor_raw, Some([9, 2, 1]));
        let gear = state.gear.as_ref().unwrap();
        assert_eq!(gear.declared_slots, 2);
        assert_eq!(gear.slots[0].item, None);
        assert_eq!(
            gear.slots[1].item.as_ref().unwrap().common.commodity_id,
            130
        );
        for length in 9..body.len() {
            assert!(world.ingest(&packet(&body[..length])).is_err());
            assert_eq!(world.entities[&42], before);
        }
        world.ingest(&packet(&[0, 0, 80, 0, 8, 1])).unwrap();
        assert_eq!(
            world.entities[&42].person.as_ref().unwrap().gear,
            state.gear
        );
    }

    #[test]
    fn sparse_updates_merge_and_truncation_is_atomic() {
        let mut world = WorldState::default();
        // Common/item flags, HP, stamina/growth, hunger and agent bits.
        let body = [0, 4, 0xff, 0xe2, 97, 0, 12, 0x80, 7, 0x21];
        world.ingest(&packet(&body)).unwrap();
        let state = world.entities[&42].person.as_ref().unwrap();
        assert_eq!(state.hp, Some(-30));
        assert_eq!(state.stamina, Some(97));
        assert_eq!(state.hunger_raw, Some(0x8007));
        assert_eq!(state.agent_bits, Some(0x21));
        world.ingest(&packet(&[0, 0, 88, 0, 8, 2])).unwrap();
        let before = world.entities[&42].clone();
        assert_eq!(before.person.as_ref().unwrap().hp, Some(-30));
        assert_eq!(before.person.as_ref().unwrap().hunger_raw, Some(0x8007));
        assert_eq!(before.person.as_ref().unwrap().agent_bits, Some(2));
        for length in 0..body.len() {
            assert!(world.ingest(&packet(&body[..length])).is_err());
            assert_eq!(world.entities[&42], before);
        }
    }

    #[test]
    fn dna_null_empty_and_unknown_layers_are_preserved() {
        let mut world = WorldState::default();
        for (length, expected) in [(u32::MAX, None), (0, Some(Vec::new()))] {
            let mut body = vec![0, 1, 0x24];
            body.extend(length.to_be_bytes());
            body.extend([127, 0, 0]);
            world.ingest(&packet(&body)).unwrap();
            assert_eq!(
                world.entities[&42].person.as_ref().unwrap().dna,
                Some(expected)
            );
        }
        let before = world.entities[&42].person.clone();
        // Unknown common layer: no attempt to seek creature/person flags.
        world.ingest(&packet(&[4, 0xff])).unwrap();
        assert_eq!(world.entities[&42].person, before);
        // Effects precede the common type body: never parse effects as a name.
        let mut effects = packet(&[1, 0xff, 0x99]);
        effects[13] = 0x10;
        world.ingest(&effects).unwrap();
        assert_eq!(world.entities[&42].person, before);
        assert_eq!(world.entities[&42].last_body, [1, 0xff, 0x99]);
        assert_eq!(world.entities[&42].name, None);
    }

    #[test]
    fn head_tilt_and_pose_data_are_not_reinterpreted() {
        let mut body = vec![0, 10];
        body.extend(0.25f32.to_be_bytes());
        body.extend(0u8..10);
        body.extend(u32::MAX.to_be_bytes());
        body.extend([127, 3, 1]); // Unknown citizen layer stops before hunger.
        let update = crate::world::decode_atom(&packet(&body)).unwrap().unwrap();
        let state = update.person.unwrap();
        assert_eq!(state.head_tilt, Some(0.25));
        assert_eq!(state.poses, Some(std::array::from_fn(|i| i as u8)));
        assert_eq!(state.pose_parameter, Some(u32::MAX));
        assert_eq!(state.growth, Some(3));
        assert_eq!(state.agent_bits, None);
        body[2..6].copy_from_slice(&f32::NAN.to_be_bytes());
        assert!(crate::world::decode_atom(&packet(&body)).is_err());
    }
}
