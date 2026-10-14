//! Established fields of an existing native registration record.
//! Not a complete record layout: mutexes, timing statistics and histories are
//! deliberately absent. Construction requires caller-established field values.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistrationFields {
    /// Hash-node offset 0x20. Existing updates do not overwrite this byte.
    pub atom_type: u8,
    /// Hash-node offset 0x3c.
    pub solar_id: u32,
    /// Raw QElapsedTimer words at hash-node offsets 0x28 and 0x30. Never treated
    /// as wall-clock timestamps or converted to Rust Instant without evidence.
    pub timer_words: [i64; 2],
    /// Hash-node byte 0x38.
    pub released: bool,
}

impl RegistrationFields {
    /// Native ReleaseTicker/ReleaseTocker field effects. Owner must also mark
    /// its corresponding cleanup-pending flag; this does not delete the record.
    pub fn release(&mut self) {
        self.released = true;
        self.timer_words = [i64::MIN; 2];
    }

    /// Existing-key UpdateOrAddTicker branch. Does not restart the timer,
    /// change atom type, or clear the owner's cleanup-pending flag.
    pub fn update_existing(&mut self, solar_id: u32) {
        self.released = false;
        self.solar_id = solar_id;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TockerFields {
    pub common: RegistrationFields,
    /// Hash-node offset 0x24; unsigned bits, units not yet established.
    pub raw_interval: u32,
}

impl TockerFields {
    /// Existing-key UpdateOrAddTocker branch; retains all timer/type fields.
    pub fn update_existing(&mut self, solar_id: u32, raw_interval: u32) {
        self.common.update_existing(solar_id);
        self.raw_interval = raw_interval;
    }

    pub fn release(&mut self) {
        self.common.release();
    }
}

#[cfg(test)]
mod tests;
