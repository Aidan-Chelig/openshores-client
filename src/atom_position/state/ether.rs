//! Fresh SetPosByEther primitive; owning AuEther orchestration remains separate.
use super::{FreshPosition, NativePositionState, PackedPosition};
use std::{io, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EtherPositionEffects {
    /// Native helper calls TickTock AFTER copying/restarting when OLD parent=0.
    /// Caller must handle this before AuEther's subsequent cache invalidation.
    pub tick_tock_required: bool,
    pub restart_movement_timer: bool,
}

impl NativePositionState {
    /// Fresh SetPosAndOrientByEther copy/extract/timer branch. The owning
    /// hierarchy still must handle TickTock and later cache invalidation.
    pub fn set_fresh_position_and_orientation_by_ether(
        &mut self,
        record: &PackedPosition,
    ) -> io::Result<EtherPositionEffects> {
        self.set_fresh_position_and_orientation_by_ether_at(record, Instant::now())
    }

    pub fn set_fresh_position_and_orientation_by_ether_at(
        &mut self,
        record: &PackedPosition,
        now: Instant,
    ) -> io::Result<EtherPositionEffects> {
        let incoming = FreshPosition::from_packed(record)?;
        let orientation = crate::atom_position::native_rotation(&incoming.local)?;
        let tick_tock_required = self.position.parent == 0;
        // Validate all modeled arithmetic before mutation (Rust safety policy).
        self.position = incoming;
        self.orientation = Some(orientation);
        self.position_update_epoch = Some(now);
        Ok(EtherPositionEffects {
            tick_tock_required,
            restart_movement_timer: true,
        })
    }

    /// Apply the established fresh-position SetPosByEther copy branch only.
    /// Caller owns prior hierarchy-index changes, TickTock and subsequent
    /// InvalidateEnvTransform. This does not fulfill ChangeParent by itself.
    /// Does not update the separate orientation scalars (SetPosAndOrient path).
    pub fn set_fresh_position_by_ether(
        &mut self,
        record: &PackedPosition,
    ) -> io::Result<EtherPositionEffects> {
        self.set_fresh_position_by_ether_at(record, Instant::now())
    }

    pub fn set_fresh_position_by_ether_at(
        &mut self,
        record: &PackedPosition,
        now: Instant,
    ) -> io::Result<EtherPositionEffects> {
        let incoming = FreshPosition::from_packed(record)?;
        let tick_tock_required = self.position.parent == 0;
        self.position = incoming;
        self.position_update_epoch = Some(now);
        // Native helper does not touch the environment-valid flag here.
        Ok(EtherPositionEffects {
            tick_tock_required,
            restart_movement_timer: true,
        })
    }
}

#[cfg(test)]
mod tests;
