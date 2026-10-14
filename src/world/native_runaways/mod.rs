//! Native AddRunaway pair collection; not yet full FixRunaways orchestration.
//! Exact pairs are unique, but the same child can await multiple parents.
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

const MAX_PAIRS: usize = 128_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRunaways {
    parents: BTreeMap<u32, BTreeSet<u32>>,
    pairs: usize,
    limit: usize,
}

impl Default for NativeRunaways {
    fn default() -> Self {
        Self {
            parents: BTreeMap::new(),
            pairs: 0,
            limit: MAX_PAIRS,
        }
    }
}

impl NativeRunaways {
    /// FixRunaways extracts the whole parent batch before object lookups.
    /// Only the recovery orchestration may consume this native queue.
    pub(crate) fn take_for_recovery(&mut self, parent: u32) -> BTreeSet<u32> {
        let children = self.parents.remove(&parent).unwrap_or_default();
        self.pairs -= children.len();
        children
    }
    /// Rust safety budget, optionally tightened; not native queue policy.
    pub fn with_limit(limit: usize) -> io::Result<Self> {
        if limit == 0 || limit > MAX_PAIRS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid runaway limit",
            ));
        }
        Ok(Self {
            limit,
            ..Self::default()
        })
    }

    /// Recovered AddRunaway(parent, child): exact duplicate is a no-op. A child
    /// under another parent is retained; zero IDs are not rejected by this path.
    /// Returns whether a new pair was inserted. Budget failures are atomic.
    pub fn add(&mut self, parent: u32, child: u32) -> io::Result<bool> {
        if !self.check_add(parent, child)? {
            return Ok(false);
        }
        self.parents.entry(parent).or_default().insert(child);
        self.pairs += 1;
        Ok(true)
    }

    pub(crate) fn check_add(&self, parent: u32, child: u32) -> io::Result<bool> {
        if self.contains(parent, child) {
            return Ok(false);
        }
        if self.pairs >= self.limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "runaways exceed pair budget",
            ));
        }
        Ok(true)
    }

    pub fn contains(&self, parent: u32, child: u32) -> bool {
        self.parents
            .get(&parent)
            .is_some_and(|children| children.contains(&child))
    }
    pub fn children(&self, parent: u32) -> Option<&BTreeSet<u32>> {
        self.parents.get(&parent)
    }
    pub fn len(&self) -> usize {
        self.pairs
    }
    pub fn is_empty(&self) -> bool {
        self.pairs == 0
    }
}

#[cfg(test)]
mod tests;
