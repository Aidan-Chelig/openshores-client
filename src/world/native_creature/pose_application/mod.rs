//! Verified pose data/application steps, not skeletal AnimatePose or SitAtConsole.
//! Required animation callbacks must implement their effect or return Unsupported.
use super::{NativeCreatureBodyKind, NativeCreaturePose};
use std::{io, time::Instant};
mod station;
pub use station::{NativeStationPoseRequest, native_station_occupant_position};
mod body;
pub use body::{NativeAnimaliaPoseChange, NativeAnimaliaPoseState, NativeDpBodyPoseState};

/// Constructor-established scalar/timer subset. Use only after fallback body
/// Pose, never to replace the separate SitAtConsole/SetStationPose effects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeCreaturePoseMarkers {
    pub field_5c0: u16,
    pub field_492: u16,
    pub last_restart: Option<Instant>,
}

impl NativeCreaturePoseMarkers {
    pub fn after_body_pose(&mut self, changed: bool, station: u16) -> bool {
        self.after_body_pose_with_clock(changed, station, Instant::now)
    }

    /// Assign5c0, restart monotonic timer, assign492, in that order. An identical
    /// pose still restarts when the station short differs. Unchanged paths do
    /// not sample the clock; wall-clock body timestamps are a separate stage.
    pub fn after_body_pose_with_clock(
        &mut self,
        changed: bool,
        station: u16,
        mut now: impl FnMut() -> Instant,
    ) -> bool {
        if !changed && self.field_5c0 == station {
            return false;
        }
        self.field_5c0 = station;
        self.last_restart = Some(now());
        self.field_492 = station;
        true
    }
}

#[cfg(test)]
mod tests;
