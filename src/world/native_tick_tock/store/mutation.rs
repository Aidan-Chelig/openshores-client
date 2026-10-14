use super::*;
use crate::world::native_tick_tock::RegistrationAction;

fn fresh(atom_type: u8, solar_id: u32) -> RegistrationFields {
    RegistrationFields {
        atom_type,
        solar_id,
        timer_words: [i64::MIN; 2],
        released: false,
    }
}

fn budget(len: usize, limit: usize) -> io::Result<()> {
    if len >= limit {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "registration table exceeds budget",
        ))
    } else {
        Ok(())
    }
}

fn sweep<T>(entries: &mut BTreeMap<u32, Entry<T>>, released: impl Fn(&T) -> bool) {
    entries.retain(|_, entry| {
        if !released(&entry.fields) {
            return true;
        }
        // Qt tries the per-record lock without waiting, unlocks, then erases.
        // Poisoned Rust locks conservatively retain entries; not native policy.
        match entry.lock.try_lock() {
            Ok(guard) => {
                drop(guard);
                false
            }
            Err(_) => true,
        }
    });
}

impl RegistrationStore {
    /// Native UpdateOrAdd metadata path. Existing entries retain their original
    /// type/timer; only missing keys trigger insertion and pending cleanup.
    /// Budget failures are a Rust boundary and leave all fields/flags unchanged.
    pub fn apply(&mut self, action: RegistrationAction) -> io::Result<()> {
        use RegistrationAction::*;
        match action {
            ReleaseTickTock { id } => {
                self.release_ticker(id);
                self.release_tocker(id);
            }
            ReleaseTicker { id } => self.release_ticker(id),
            ReleaseTocker { id } => self.release_tocker(id),
            UpdateOrAddTicker {
                id,
                atom_type,
                solar_id,
            } => {
                if let Some(entry) = self.tickers.get_mut(&id) {
                    entry.fields.update_existing(solar_id);
                } else {
                    budget(self.tickers.len(), self.limit)?;
                    self.tickers.insert(
                        id,
                        Entry {
                            fields: fresh(atom_type, solar_id),
                            lock: Arc::new(Mutex::new(())),
                        },
                    );
                    if self.ticker_cleanup_pending {
                        sweep(&mut self.tickers, |fields| fields.released);
                        // Native clears this flag even when a busy record survives.
                        self.ticker_cleanup_pending = false;
                    }
                }
            }
            UpdateOrAddTocker {
                id,
                atom_type,
                solar_id,
                raw_interval,
            } => {
                if let Some(entry) = self.tockers.get_mut(&id) {
                    entry.fields.update_existing(solar_id, raw_interval);
                } else {
                    budget(self.tockers.len(), self.limit)?;
                    self.tockers.insert(
                        id,
                        Entry {
                            fields: TockerFields {
                                common: fresh(atom_type, solar_id),
                                raw_interval,
                            },
                            lock: Arc::new(Mutex::new(())),
                        },
                    );
                    if self.tocker_cleanup_pending {
                        sweep(&mut self.tockers, |fields| fields.common.released);
                        self.tocker_cleanup_pending = false;
                    }
                }
            }
        }
        Ok(())
    }

    fn release_ticker(&mut self, id: u32) {
        if let Some(entry) = self.tickers.get_mut(&id) {
            entry.fields.release();
            self.ticker_cleanup_pending = true;
        }
    }
    fn release_tocker(&mut self, id: u32) {
        if let Some(entry) = self.tockers.get_mut(&id) {
            entry.fields.release();
            self.tocker_cleanup_pending = true;
        }
    }
}
