use super::*;

/// Required application effects, never optional no-op callbacks. Implementors
/// must perform the native operation or return Unsupported. Trace-only fixture
/// contexts establish ordering, not complete Creature/Person integration.
pub trait NativeCreatureReceiveContext {
    /// Ensure/create selected model first, then destroy the other model. Called
    /// for every update, even without DNA/pose. `created` concerns receive state;
    /// a concrete context must also know its actual native-model lifecycle.
    fn select_body(&mut self, kind: NativeCreatureBodyKind, created: bool) -> io::Result<()>;

    /// DpBody byte3a=bit2 and byte64=inverse bit0 BEFORE reading its DhDNA body.
    fn configure_dp_body(&mut self, fields_3a_64: [u8; 2]) -> io::Result<()>;

    /// DaCreature::SetDNA and model effects. Animalia additionally calls its
    /// SetDNA with configuration bit0, then SetLeftHanded with bit2. These
    /// setters can initialize poses/restart timers; storing DNA is insufficient.
    fn set_dna(&mut self, dna: &NativeDna, configuration: u8) -> io::Result<()>;

    /// The real virtual setter at +1e0, not just its last received float value.
    fn set_scalar_1e0(&mut self, value: f32) -> io::Result<()>;

    fn set_hit_points(&mut self, kind: NativeCreatureBodyKind, clamped: i16) -> io::Result<()>;

    /// Sample before pose. Return0x3d if there is no current console pose;
    /// otherwise actual console +f0, not an inferred packet type.
    fn old_console_type(&mut self) -> io::Result<u32>;

    /// Apply SetStationPose first (DpBody uses words starting at index3,
    /// Animalia index0), then fallback body Pose/movement/timer effects. The
    /// typed wire pose is supplied; this method must not read extra pose bytes.
    fn apply_pose(
        &mut self,
        kind: NativeCreatureBodyKind,
        pose: &NativeCreaturePose,
        state: &NativeCreatureReceiveState,
    ) -> io::Result<()>;

    /// Preserve matching-console/context lookup, temporary factory, settings
    /// and local augmentation semantics. None means no console flag and still
    /// requires native conditional console destruction/retention. Some(type)
    /// may consume a concrete station reader only when native context allows.
    /// Unknown/unsupported bodies must never be skipped or source-decoded.
    fn receive_console(
        &mut self,
        received_type: Option<u8>,
        old_type: u32,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<()>;

    fn wall_clock_ms(&mut self) -> io::Result<i64>;

    fn set_body_timestamp(
        &mut self,
        kind: NativeCreatureBodyKind,
        milliseconds: i64,
    ) -> io::Result<()>;

    /// Actual parent vehicle SeatAt(current location), or zero for no vehicle.
    /// Evaluated after optional 5b8/5bc reception, not another packet byte.
    fn seat_at_current_location(&mut self) -> io::Result<u8>;
}
