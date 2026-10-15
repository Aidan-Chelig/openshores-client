//! Declared fixture observations/effects, not the client's animation system.
use crate::world::native_creature::{NativeHeadTiltContext, NativeHeadTiltState};
use std::{io, time::Instant};
pub(super) struct HeadEffects;
impl NativeHeadTiltContext for HeadEffects {
    fn movement_speed_modifier(&mut self) -> io::Result<f32> {
        Ok(0.25)
    }
    fn monotonic_now(&mut self) -> Instant {
        Instant::now()
    }
    fn parent_is_creature(&mut self) -> io::Result<bool> {
        Ok(false)
    }
    fn parent_is_vehicle(&mut self) -> io::Result<bool> {
        Ok(false)
    }
    fn pointer_690_nonnull(&mut self) -> io::Result<bool> {
        Ok(false)
    }
    fn animate_dp(&mut self, _: &NativeHeadTiltState, mode: bool, speed: f32) -> io::Result<()> {
        assert!(!mode);
        assert_eq!(speed, 0.25);
        Ok(())
    }
    fn animate_animalia(&mut self, _: &NativeHeadTiltState, _: Instant) -> io::Result<()> {
        Ok(())
    }
    fn set_in_motion(&mut self) -> io::Result<()> {
        Ok(())
    }
}
