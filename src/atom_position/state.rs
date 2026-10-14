//! Fresh wire-position branch of native ChangePos. Hierarchy effects stay explicit.
use super::PackedPosition;
use std::io;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct FreshPosition {
    pub parent: u32,
    pub local: [[f64; 4]; 4],
}

impl FreshPosition {
    /// RxUpdateBase constructs a fresh AuAtomPos with null parent before Rx.
    /// This is not Rx applied to an arbitrary preexisting matrix/parent.
    pub fn from_packed(record: &PackedPosition) -> io::Result<Self> {
        Ok(Self {
            parent: record.parent.unwrap_or(0),
            local: record.local_matrix()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PositionChange {
    /// Caller must run the hierarchy-manager path; no local mutation happened.
    ReparentRequired(FreshPosition),
    /// Native local-copy branch. Timer restarted; caller owns descendant caches.
    Applied {
        restart_movement_timer: bool,
        invalidate_normal_descendants: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativePositionState {
    pub position: FreshPosition,
    pub environment_cache_valid: bool,
    position_update_epoch: Option<Instant>,
}

impl Default for NativePositionState {
    fn default() -> Self {
        Self {
            position: FreshPosition {
                parent: 0,
                local: std::array::from_fn(|column| {
                    std::array::from_fn(|row| f64::from(column == row))
                }),
            },
            environment_cache_valid: false,
            position_update_epoch: None,
        }
    }
}

impl NativePositionState {
    /// Apply a decoded fresh wire position only to the established local branch.
    /// An owning hierarchy with a changed parent delegates instead of copying.
    /// Timer restart is owned here; normal-descendant invalidation remains a
    /// caller obligation. Full AuEther::ChangeParent is not implemented here.
    pub fn apply_fresh_wire_position(
        &mut self,
        record: &PackedPosition,
        has_owning_hierarchy: bool,
    ) -> io::Result<PositionChange> {
        self.apply_fresh_wire_position_at(record, has_owning_hierarchy, Instant::now())
    }

    /// Inject the monotonic update instant for deterministic replay/tests.
    /// Managed reparent requests and failed validation do not restart the timer.
    pub fn apply_fresh_wire_position_at(
        &mut self,
        record: &PackedPosition,
        has_owning_hierarchy: bool,
        now: Instant,
    ) -> io::Result<PositionChange> {
        let incoming = FreshPosition::from_packed(record)?;
        if has_owning_hierarchy && self.position.parent != incoming.parent {
            return Ok(PositionChange::ReparentRequired(incoming));
        }
        let invalidate_normal_descendants = self.environment_cache_valid;
        self.position = incoming;
        self.position_update_epoch = Some(now);
        self.environment_cache_valid = false;
        Ok(PositionChange::Applied {
            restart_movement_timer: true,
            invalidate_normal_descendants,
        })
    }

    /// Age since a successful local-copy update. None before the first known
    /// update or if the supplied clock precedes it; no wall-clock arithmetic.
    pub fn position_update_age_at(&self, now: Instant) -> Option<Duration> {
        now.checked_duration_since(self.position_update_epoch?)
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
