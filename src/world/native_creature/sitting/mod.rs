//! Concrete sitting pose/console state transitions. External UI/capsule/render
//! effects remain required context operations, never implicit success no-ops.
use super::*;
use std::time::Instant;
mod apply;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeSittingRequest {
    pub station: u16,
    pub station_type: u32,
    pub occupant_position: u32,
    pub variant: i32,
    pub trailing_station: u16,
}

impl From<NativeStationPoseRequest> for NativeSittingRequest {
    fn from(value: NativeStationPoseRequest) -> Self {
        Self {
            station: value.station,
            station_type: value.station_type,
            occupant_position: u32::from(value.occupant_position),
            variant: value.variant as i32,
            trailing_station: value.trailing_station,
        }
    }
}

/// Native constructor-established pose/console subset, not the full Creature
/// model, inherited state or native object constructor. DpBody starts selected.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeSittingState {
    pub dp_body: Option<NativeDpBodyPoseState>,
    pub animalia: Option<NativeAnimaliaPoseState>,
    pub console: Option<NativeConsoleState>,
    pub markers: NativeCreaturePoseMarkers,
}
impl Default for NativeSittingState {
    fn default() -> Self {
        Self {
            dp_body: Some(NativeDpBodyPoseState::default()),
            animalia: None,
            console: None,
            markers: NativeCreaturePoseMarkers::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeSittingChange {
    Created,
    Replaced,
    Reused,
    Cleared,
    Absent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeSittingDiagnostic {
    IntegratedDesigner,
    UnknownStationType(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "inspect diagnostic and transition results; this is not complete Person reception"]
pub struct NativeSittingResult {
    pub general_pose: u32,
    pub console_change: NativeSittingChange,
    pub body_changed: bool,
    pub timer_restarted: bool,
    pub diagnostic: Option<NativeSittingDiagnostic>,
}

pub trait NativeSittingContext {
    type Capsule;
    /// Immediate parent dynamic-cast to DaCapsule, not the separate +468 lookup
    /// and not an arbitrary ancestor. Required even for non-console stations.
    fn parent_capsule(&mut self) -> io::Result<Option<Self::Capsule>>;
    fn destroy_console(&mut self, console: NativeConsoleState) -> io::Result<()>;
    /// Rust constructor subset is supplied; real UI/resource setup must succeed
    /// or fail explicitly before the new console is installed.
    fn create_console(&mut self, console: &NativeConsoleState) -> io::Result<()>;
    /// Parent capsule virtual+d48 with this actor's identity/context. Invoked
    /// after installing/reusing a console, before model pose application.
    fn notify_parent_capsule(
        &mut self,
        capsule: &Self::Capsule,
        console: &NativeConsoleState,
    ) -> io::Result<()>;
    fn dp_model_data_nonempty(&mut self) -> io::Result<bool>;
    fn animate_dp(&mut self, state: &NativeDpBodyPoseState, speed: f32) -> io::Result<()>;
    fn animate_animalia(&mut self, state: &NativeAnimaliaPoseState) -> io::Result<()>;
    fn monotonic_now(&mut self) -> Instant;
}

#[cfg(test)]
mod tests;
