use super::*;

impl NativeSittingState {
    pub fn current_general_pose(&self) -> io::Result<u32> {
        native_current_general_pose(self.dp_body.as_ref(), self.animalia.as_ref())
    }

    /// SetStationPose chooses the body-dependent incoming word. None means
    /// caller must execute the real non-station body fallback, not skip pose.
    pub fn set_station_pose(
        &mut self,
        pose: &NativeCreaturePose,
        mode: bool,
        random_state: Option<&mut u32>,
        context: &mut impl NativeSittingContext,
    ) -> io::Result<Option<NativeSittingResult>> {
        let kind = if self.dp_body.is_some() {
            NativeCreatureBodyKind::DpBody
        } else if self.animalia.is_some() {
            NativeCreatureBodyKind::Animalia
        } else {
            return Err(no_body());
        };
        let Some(request) = NativeStationPoseRequest::from_received_pose(kind, pose) else {
            return Ok(None);
        };
        self.sit_at_console(request.into(), mode, random_state, context)
            .map(Some)
    }

    /// Compose real OpPose, general-pose-only reuse, constructors, notification,
    /// body Pose and console markers. Required external effects must succeed or
    /// return Unsupported. Earlier completed transitions survive later errors.
    /// Rust error policy is not native exception/Qt-lock/destructor parity.
    pub fn sit_at_console(
        &mut self,
        request: NativeSittingRequest,
        mode: bool,
        random_state: Option<&mut u32>,
        context: &mut impl NativeSittingContext,
    ) -> io::Result<NativeSittingResult> {
        let parent = context.parent_capsule()?;
        let general_pose = native_station_operator_pose(
            request.station_type,
            request.occupant_position,
            request.variant,
            random_state,
        )?;
        let console_kind = matches!(request.station_type,0..=5|7..=0x10|0x12..=0x15|0x21..=0x23);
        let console_change = if console_kind {
            if self.console.is_some() && self.current_general_pose()? == general_pose {
                NativeSittingChange::Reused
            } else {
                let replaced = self.destroy_console(context)?;
                let console = NativeConsoleState::from_native_constructor(
                    request.station_type as u8,
                    request.trailing_station,
                )?;
                context.create_console(&console)?;
                self.console = Some(console);
                if replaced {
                    NativeSittingChange::Replaced
                } else {
                    NativeSittingChange::Created
                }
            }
        } else if self.destroy_console(context)? {
            NativeSittingChange::Cleared
        } else {
            NativeSittingChange::Absent
        };
        if let (Some(parent), Some(console)) = (parent.as_ref(), self.console.as_ref()) {
            context.notify_parent_capsule(parent, console)?;
        }
        let body_changed = if let Some(body) = self.dp_body.as_mut() {
            // Guard unknown external model state before applying the pose. This
            // early error boundary is Rust policy, not native failed-call parity.
            let nonempty = context.dp_model_data_nonempty()?;
            body.apply(
                native_station_body_pose(NativeCreatureBodyKind::DpBody, general_pose),
                mode,
                1.,
                nonempty,
                |state, speed| context.animate_dp(state, speed),
            )?
        } else if let Some(body) = self.animalia.as_mut() {
            body.apply(
                native_station_body_pose(NativeCreatureBodyKind::Animalia, general_pose),
                mode,
                1.,
                |state| context.animate_animalia(state),
            )?
            .changed
        } else {
            return Err(no_body());
        };
        let timer_restarted = self.markers.after_console_pose_with_clock(
            body_changed,
            request.station,
            request.trailing_station,
            || context.monotonic_now(),
        );
        Ok(NativeSittingResult {
            general_pose,
            console_change,
            body_changed,
            timer_restarted,
            diagnostic: if request.station_type == 0x3c {
                Some(NativeSittingDiagnostic::IntegratedDesigner)
            } else if request.station_type > 0x3c {
                Some(NativeSittingDiagnostic::UnknownStationType(
                    request.station_type,
                ))
            } else {
                None
            },
        })
    }

    fn destroy_console(&mut self, context: &mut impl NativeSittingContext) -> io::Result<bool> {
        if let Some(console) = self.console.take() {
            context.destroy_console(console)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn no_body() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "native sitting requires a selected body",
    )
}
