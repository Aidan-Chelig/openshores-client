use crate::protocol::QtReader;
use std::{collections::BTreeMap, io};

/// Raw skill IDs and values, without guessed enums or native hash iteration order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeSkillSet(pub BTreeMap<u16, i32>);

impl NativeSkillSet {
    pub fn receive(&mut self, input: &mut QtReader<'_>) -> io::Result<()> {
        input.read_u8()?; // Received but not stored by the native reader.
        self.0.clear();
        for _ in 0..input.read_u16()? {
            let skill = input.read_u16()?;
            let value = input.read_i32()?;
            self.0.insert(skill, value);
        }
        Ok(())
    }
}
