//! Person virtual+1e0 -> DaCreature::TiltHeadTo(float).
use super::NativeCreatureBodyKind;
use std::{io, time::Instant};

/// Stored DpBody30/34 or Animalia64/68. Both constructors establish zero.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeHeadTiltState {
    pub vertical: f32,
    pub horizontal: f32,
    pub timer_5e8: Option<Instant>,
}

pub trait NativeHeadTiltContext {
    /// Actual growth/HP/burden-dependent modifier, before angle mutation. No
    /// guessed constant speed is a valid implementation of this operation.
    fn movement_speed_modifier(&mut self) -> io::Result<f32>;
    fn monotonic_now(&mut self) -> Instant;
    fn parent_is_creature(&mut self) -> io::Result<bool>;
    fn parent_is_vehicle(&mut self) -> io::Result<bool>;
    fn pointer_690_nonnull(&mut self) -> io::Result<bool>;
    fn animate_dp(&mut self, head: &NativeHeadTiltState, mode: bool, speed: f32) -> io::Result<()>;
    /// Native starts a fresh elapsed timer for Animalia::AnimatePose. It is
    /// distinct from Creature5e8, even when the clock samples happen to match.
    fn animate_animalia(&mut self, head: &NativeHeadTiltState, started: Instant) -> io::Result<()>;
    fn set_in_motion(&mut self) -> io::Result<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "angle storage alone does not establish animation/presentation"]
pub struct NativeHeadTiltResult {
    pub changed: bool,
    pub animated: bool,
}

/// AuPose::IsLocked's unsigned comparisons, not a guessed pose category enum.
pub fn native_pose_is_locked(word: u32) -> bool {
    matches!(word, 3 | 5..=15 | 57..=73 | 76..=135)
}

impl NativeHeadTiltState {
    /// Ordered head-angle/timer/animation effects. The caller must supply the
    /// selected body's real pose words and actual runtime context. Native locks
    /// and concurrent replacement are not emulated by this exclusive Rust call.
    /// Earlier writes survive later errors; unsupported animation is not success.
    pub fn apply(
        &mut self,
        requested: f32,
        kind: NativeCreatureBodyKind,
        pose: &[u32; 10],
        context: &mut impl NativeHeadTiltContext,
    ) -> io::Result<NativeHeadTiltResult> {
        let general = pose[if kind == NativeCreatureBodyKind::DpBody {
            3
        } else {
            0
        }];
        let value = if (general == 0x39 && requested > 0.) || (general == 0x0f && requested < 0.) {
            0.
        } else {
            requested
        };
        // Compare before the body angle clamp: repeated out-of-range requests
        // still enter the mutation path. Equal signed zeros do not; NaNs do.
        if value == self.vertical {
            return Ok(NativeHeadTiltResult {
                changed: false,
                animated: false,
            });
        }
        let speed = context.movement_speed_modifier()?;
        let upper = f32::from_bits(0x3fc9_0fdb);
        let lower = f32::from_bits(0xbfc9_0fdb);
        // MAXSS uses requested as second operand, preserving its NaN/signed zero.
        self.vertical = if value > upper {
            upper
        } else if value < lower {
            lower
        } else {
            value
        };
        self.horizontal = 0.;
        self.timer_5e8 = Some(context.monotonic_now());
        let pose_suppresses = general.wrapping_sub(0x4a) <= 1
            || match kind {
                NativeCreatureBodyKind::DpBody => [pose[1], pose[6], pose[8]].contains(&0x17),
                NativeCreatureBodyKind::Animalia => pose[3] == 0x17,
            };
        if native_pose_is_locked(general) || pose_suppresses {
            return Ok(NativeHeadTiltResult {
                changed: true,
                animated: false,
            });
        }
        if context.parent_is_creature()? || context.parent_is_vehicle()? {
            return Ok(NativeHeadTiltResult {
                changed: true,
                animated: false,
            });
        }
        match kind {
            NativeCreatureBodyKind::DpBody => {
                let mode = context.pointer_690_nonnull()?;
                context.animate_dp(self, mode, speed)?;
            }
            NativeCreatureBodyKind::Animalia => {
                let started = context.monotonic_now();
                context.animate_animalia(self, started)?;
            }
        }
        context.set_in_motion()?;
        Ok(NativeHeadTiltResult {
            changed: true,
            animated: true,
        })
    }
}

#[cfg(test)]
mod tests;
