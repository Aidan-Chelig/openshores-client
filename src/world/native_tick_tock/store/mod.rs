//! Owned registration metadata and deferred cleanup, not a scheduler loop.
//! BTree order and Rust mutex poisoning are not native QHash/Qt lock parity.
mod integration;
mod mutation;
use super::record::{RegistrationFields, TockerFields};
pub use integration::StoreContext;
use std::{
    collections::BTreeMap,
    io,
    sync::{Arc, Mutex},
};

pub(super) struct Entry<T> {
    pub fields: T,
    pub lock: Arc<Mutex<()>>,
}

pub struct RegistrationStore {
    tickers: BTreeMap<u32, Entry<RegistrationFields>>,
    tockers: BTreeMap<u32, Entry<TockerFields>>,
    ticker_cleanup_pending: bool,
    tocker_cleanup_pending: bool,
    limit: usize,
}

impl Default for RegistrationStore {
    fn default() -> Self {
        Self {
            tickers: BTreeMap::new(),
            tockers: BTreeMap::new(),
            ticker_cleanup_pending: false,
            tocker_cleanup_pending: false,
            limit: 20_000,
        }
    }
}

impl RegistrationStore {
    /// Static InTickTockQueue membership: native checks ticker key presence,
    /// then tocker key presence. Released records still protect an object until
    /// actually swept; this does not test release markers or execute scheduling.
    /// GNU full-function review corroborated by retained REA pseudocode.
    pub fn in_tick_tock_queue(&self, id: u32) -> bool {
        self.tickers.contains_key(&id) || self.tockers.contains_key(&id)
    }
    /// Tighten the Rust per-table safety budget, never enlarge it.
    pub fn with_limit(limit: usize) -> io::Result<Self> {
        if limit == 0 || limit > 20_000 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid registration limit",
            ));
        }
        Ok(Self {
            limit,
            ..Self::default()
        })
    }

    pub fn ticker(&self, id: u32) -> Option<&RegistrationFields> {
        self.tickers.get(&id).map(|entry| &entry.fields)
    }
    pub fn tocker(&self, id: u32) -> Option<&TockerFields> {
        self.tockers.get(&id).map(|entry| &entry.fields)
    }
    /// A scheduler owner can hold this mutex while using the registration.
    /// Released entries are retained while try_lock reports busy (or poisoned).
    pub fn ticker_lock(&self, id: u32) -> Option<Arc<Mutex<()>>> {
        self.tickers.get(&id).map(|entry| Arc::clone(&entry.lock))
    }
    pub fn tocker_lock(&self, id: u32) -> Option<Arc<Mutex<()>>> {
        self.tockers.get(&id).map(|entry| Arc::clone(&entry.lock))
    }
    pub fn cleanup_pending(&self) -> (bool, bool) {
        (self.ticker_cleanup_pending, self.tocker_cleanup_pending)
    }
}

#[cfg(test)]
mod tests;
