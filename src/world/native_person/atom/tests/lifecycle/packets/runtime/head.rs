//! Declared fixture observations/effects, not the client's animation system.
use crate::world::native_creature::{
    NativeCreatureBodyKind, NativeHeadTiltContext, NativeHeadTiltState, NativeMovementBody,
    NativeMovementContext, native_movement_speed_modifier,
};
use std::{io, time::Instant};
pub(super) struct HeadEffects {
    pub hp: i16,
    pub kind: NativeCreatureBodyKind,
}
// This fixture's packets omit growth updates, so prior494 is constructor zero.
// Model capacity/burden dependencies remain explicitly unsupported, not guessed.
impl NativeMovementContext for HeadEffects {
    fn body(&mut self) -> io::Result<NativeMovementBody> {
        Ok(match self.kind {
            NativeCreatureBodyKind::DpBody => NativeMovementBody::DpBody {
                hit_points: self.hp,
            },
            NativeCreatureBodyKind::Animalia => NativeMovementBody::Animalia {
                hit_points: self.hp,
            },
        })
    }
    fn growth_byte_494(&mut self) -> io::Result<u8> {
        Ok(0)
    }
    fn growth_time(&mut self) -> io::Result<u8> {
        Err(io::ErrorKind::Unsupported.into())
    }
    fn dp_byte_29(&mut self) -> io::Result<u8> {
        Err(io::ErrorKind::Unsupported.into())
    }
    fn max_hit_points(&mut self) -> io::Result<i16> {
        Err(io::ErrorKind::Unsupported.into())
    }
    fn burden(&mut self) -> io::Result<f32> {
        Err(io::ErrorKind::Unsupported.into())
    }
}
impl NativeHeadTiltContext for HeadEffects {
    fn movement_speed_modifier(&mut self) -> io::Result<f32> {
        native_movement_speed_modifier(self)
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
        assert_eq!(speed, 1.); // scalar stage precedes the packet's HP update
        Ok(())
    }
    fn animate_animalia(&mut self, _: &NativeHeadTiltState, _: Instant) -> io::Result<()> {
        Ok(())
    }
    fn set_in_motion(&mut self) -> io::Result<()> {
        Ok(())
    }
}
