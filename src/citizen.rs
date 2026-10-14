//! Source-established player bootstrap citizen profiles, not all AuCitizen uses.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CitizenPrefix {
    pub flags: u8,
    pub home_system: Option<u32>,
    pub home_name: Option<Option<String>>,
    /// Raw source f32 coordinates; units are not converted here.
    pub home_coordinates: Option<[f32; 2]>,
    pub empire: Option<u32>,
    pub office_tag: Option<u8>,
    pub trailing_word_raw: Option<i32>,
}

impl CitizenPrefix {
    pub fn merge(&mut self, update: Self) {
        self.flags = update.flags;
        macro_rules! merge { ($($field:ident),*) => { $(if update.$field.is_some() { self.$field = update.$field; })* }; }
        merge!(
            home_system,
            home_name,
            home_coordinates,
            empire,
            office_tag,
            trailing_word_raw
        );
    }
    pub(crate) fn read(input: &mut QtReader<'_>) -> io::Result<(Self, bool)> {
        let flags = input.read_u8()?;
        let mut result = Self {
            flags,
            ..Self::default()
        };
        // connection.py::_pb2 emits the same head for these two profiles;
        // 0x6c also announces a job blob later, after gear.
        if !matches!(flags, 0x68 | 0x6c) {
            return Ok((result, false));
        }
        result.home_system = Some(input.read_u32()?);
        result.home_name = Some(input.read_string()?);
        let coordinates = [input.read_f32()?, input.read_f32()?];
        if coordinates.iter().any(|value| !value.is_finite()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-finite citizen home coordinates",
            ));
        }
        result.home_coordinates = Some(coordinates);
        result.empire = Some(input.read_u32()?);
        let tag = input.read_u8()?;
        result.office_tag = Some(tag);
        if tag != 0 {
            return Ok((result, false));
        }
        let word = input.read_u32()? as i32;
        result.trailing_word_raw = Some(word);
        // Only the emitted zero form establishes the following field boundary.
        Ok((result, word == 0))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerStatistics {
    pub xp: i32,
    pub reputation: i32,
    pub word_raw: i16,
    pub bank: f64,
    pub second_balance_raw: f64,
    pub tag_raw: u8,
}

impl PlayerStatistics {
    pub(crate) fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let result = Self {
            xp: input.read_u32()? as i32,
            reputation: input.read_u32()? as i32,
            word_raw: input.read_u16()? as i16,
            bank: input.read_f64()?,
            second_balance_raw: input.read_f64()?,
            tag_raw: input.read_u8()?,
        };
        if !result.bank.is_finite() || !result.second_balance_raw.is_finite() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-finite player statistics balance",
            ));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{protocol::QtWriter, world::WorldState};

    fn prefix(flags: u8) -> Vec<u8> {
        let mut out = vec![flags];
        out.extend(42u32.to_be_bytes());
        let mut name = QtWriter::default();
        name.write_string(Some("Home 🚀")).unwrap();
        out.extend(name.into_bytes());
        out.extend(1.25f32.to_be_bytes());
        out.extend((-2.5f32).to_be_bytes());
        out.extend(55u32.to_be_bytes());
        out.push(0);
        out.extend(0i32.to_be_bytes());
        out
    }
    fn packet(body: &[u8]) -> Vec<u8> {
        let mut out = vec![0x12];
        out.extend(42u32.to_be_bytes());
        out.extend(1000i64.to_be_bytes());
        out.push(0);
        out.extend(body);
        out
    }
    fn bootstrap(flags: u8) -> Vec<u8> {
        let mut out = vec![0, 0, 90, 0, 0xed];
        out.extend(prefix(flags));
        out.extend(123u16.to_be_bytes());
        out.extend([5, 9, 2, 1, 0]); // agent, cursor, empty gear batch
        out.extend((-3i32).to_be_bytes());
        out.extend(7i32.to_be_bytes());
        out.extend((-2i16).to_be_bytes());
        out.extend(123.5f64.to_be_bytes());
        out.extend((-4.25f64).to_be_bytes());
        out.push(0);
        out
    }

    #[test]
    fn full_profile_unlocks_gear_and_statistics_atomically() {
        let body = bootstrap(0x68);
        let mut world = WorldState::default();
        world.ingest(&packet(&body)).unwrap();
        let before = world.entities[&42].clone();
        let state = before.person.as_ref().unwrap();
        let citizen = state.citizen.as_ref().unwrap();
        assert_eq!(citizen.home_system, Some(42));
        assert_eq!(citizen.home_name, Some(Some("Home 🚀".into())));
        assert_eq!(citizen.home_coordinates, Some([1.25, -2.5]));
        assert_eq!(citizen.empire, Some(55));
        assert_eq!(state.hunger_raw, Some(123));
        assert_eq!(state.cursor_raw, Some([9, 2, 1]));
        assert!(state.gear.as_ref().unwrap().slots.is_empty());
        assert_eq!(state.statistics.as_ref().unwrap().xp, -3);
        assert_eq!(state.statistics.as_ref().unwrap().bank, 123.5);
        for length in 0..body.len() {
            assert!(world.ingest(&packet(&body[..length])).is_err());
            assert_eq!(world.entities[&42], before);
        }
        let mut nan = body.clone();
        let bank = nan.len() - 17;
        nan[bank..bank + 8].copy_from_slice(&f64::NAN.to_be_bytes());
        assert!(world.ingest(&packet(&nan)).is_err());
        assert_eq!(world.entities[&42], before);
        world.ingest(&packet(&[0, 0, 80, 0, 1, 0xff])).unwrap();
        assert_eq!(
            world.entities[&42]
                .person
                .as_ref()
                .unwrap()
                .citizen
                .as_ref()
                .unwrap()
                .home_system,
            Some(42)
        );
        assert_eq!(
            world.entities[&42].person.as_ref().unwrap().statistics,
            state.statistics
        );
    }

    #[test]
    fn unsupported_offices_and_jobs_do_not_misalign_later_fields() {
        let mut body = bootstrap(0x6c);
        let statistics = body.len() - 27;
        body.splice(statistics..statistics, [0, 1, 0xfe]);
        let update = crate::world::decode_atom(&packet(&body)).unwrap().unwrap();
        let state = update.person.unwrap();
        assert!(state.gear.is_some());
        assert_eq!(state.statistics, None);
        let mut bytes = prefix(0x68);
        let tag = bytes.len() - 5;
        bytes[tag] = 1;
        let (citizen, complete) = CitizenPrefix::read(&mut QtReader::new(&bytes)).unwrap();
        assert!(!complete);
        assert_eq!(citizen.office_tag, Some(1));
        assert_eq!(citizen.trailing_word_raw, None);
        let mut bytes = prefix(0x68);
        let word = bytes.len() - 4;
        bytes[word..].copy_from_slice(&(-1i32).to_be_bytes());
        let (citizen, complete) = CitizenPrefix::read(&mut QtReader::new(&bytes)).unwrap();
        assert!(!complete);
        assert_eq!(citizen.trailing_word_raw, Some(-1));
        let mut bytes = prefix(0x68);
        let coordinate = bytes.len() - 17;
        bytes[coordinate..coordinate + 4].copy_from_slice(&f32::NAN.to_be_bytes());
        assert!(CitizenPrefix::read(&mut QtReader::new(&bytes)).is_err());
    }
}
