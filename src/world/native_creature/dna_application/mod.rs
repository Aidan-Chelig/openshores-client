//! DaCreature::SetDNA ordered model effects, not nested morphology generation.
use super::{NativeCreatureBodyKind, native_animalia_initialize_poses};
use crate::world::native_dna::{NativeAuDna, NativeDhDna, NativeDna};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeDpDnaPrior {
    pub byte_3a: u8,
    pub byte_64: u8,
    pub hit_points: i16,
    pub general_pose_48: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeAnimaliaDnaPrior {
    pub chromatid_08: u32,
    pub handedness_2c: u32,
    pub hit_points: i16,
    pub general_pose_6c: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeDnaModelSnapshot {
    pub dp: Option<NativeDpDnaPrior>,
    pub animalia: Option<NativeAnimaliaDnaPrior>,
    pub creature_byte_494: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeDnaTimer {
    Field5c8,
    Field630,
}

/// Actual model operations must succeed or return Unsupported. No default
/// implementation substitutes received DNA storage for native geometry/effects.
pub trait NativeDnaModelContext {
    /// Snapshot actual model fields consistently; not packet defaults. The
    /// exclusive Rust call has no intervening external model replacement.
    fn snapshot(&mut self) -> io::Result<NativeDnaModelSnapshot>;
    fn create_animalia(
        &mut self,
        dna: &NativeAuDna,
        chromatid: u32,
        handedness: u32,
        growth_stage: u32,
    ) -> io::Result<()>;
    fn update_animalia(&mut self, dna: &NativeAuDna, chromatid: u32) -> io::Result<()>;
    fn create_dp(
        &mut self,
        dna: &NativeDhDna,
        first_char_parameter: u8,
        second_bool_parameter: bool,
    ) -> io::Result<()>;
    fn update_dp(&mut self, dna: &NativeDhDna, scale: f32) -> io::Result<()>;
    fn set_hit_points(&mut self, kind: NativeCreatureBodyKind, clamped: i16) -> io::Result<()>;
    fn destroy_dp(&mut self) -> io::Result<()>;
    fn destroy_animalia(&mut self) -> io::Result<()>;
    /// Apply real Animalia::Pose, including its animation operation.
    fn animalia_pose(&mut self, fields: [u32; 10], mode: bool, speed: f32) -> io::Result<()>;
    fn restart_timer(&mut self, timer: NativeDnaTimer) -> io::Result<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "model transition result is not complete Creature or Person reception"]
pub struct NativeCreatureDnaApplication {
    pub kind: NativeCreatureBodyKind,
    pub created: bool,
    pub removed_other_body: bool,
    pub pose_reinitialized: bool,
}

/// Real outer SetDNA stages. Typed DNA replaces native dynamic casts, including
/// the DhDNA bad_cast catch branch verified in independent assembly. Nested
/// constructors/SetDNA/Pose remain required actual operations. Earlier effects
/// survive later Rust errors; Qt locks/C++ exception recovery are not emulated.
pub fn native_apply_creature_dna(
    dna: &NativeDna,
    context: &mut impl NativeDnaModelContext,
) -> io::Result<NativeCreatureDnaApplication> {
    let old = context.snapshot()?;
    let (chromatid, handedness, hp) = if let Some(animal) = old.animalia {
        (animal.chromatid_08, animal.handedness_2c, animal.hit_points)
    } else if let Some(dp) = old.dp {
        // DpBody::Chromatid compares the full byte with 1, not its low bit.
        (
            u32::from(dp.byte_64 != 1),
            u32::from(dp.byte_3a ^ 1),
            dp.hit_points,
        )
    } else {
        (0, 1, 0)
    };
    match dna {
        NativeDna::Au(dna) => {
            let general_pose = if let Some(animal) = old.animalia {
                context.update_animalia(dna, chromatid)?;
                animal.general_pose_6c
            } else {
                let pose = old.dp.map_or(0x24, |dp| dp.general_pose_48);
                context.create_animalia(
                    dna,
                    chromatid,
                    handedness,
                    u32::from(old.creature_byte_494 == 0),
                )?;
                pose
            };
            context.set_hit_points(NativeCreatureBodyKind::Animalia, hp.max(-30))?;
            if old.dp.is_some() {
                context.destroy_dp()?;
            }
            context.restart_timer(NativeDnaTimer::Field5c8)?;
            context.animalia_pose(native_animalia_initialize_poses(general_pose), false, 1.)?;
            context.restart_timer(NativeDnaTimer::Field630)?;
            Ok(NativeCreatureDnaApplication {
                kind: NativeCreatureBodyKind::Animalia,
                created: old.animalia.is_none(),
                removed_other_body: old.dp.is_some(),
                pose_reinitialized: true,
            })
        }
        NativeDna::Dh(dna) => {
            if old.dp.is_none() {
                context.create_dp(dna, u8::from(chromatid == 0), handedness == 0)?;
            } else {
                context.update_dp(dna, 1.)?;
            }
            context.set_hit_points(NativeCreatureBodyKind::DpBody, hp.max(-30))?;
            if old.animalia.is_some() {
                context.destroy_animalia()?;
            }
            context.restart_timer(NativeDnaTimer::Field5c8)?;
            Ok(NativeCreatureDnaApplication {
                kind: NativeCreatureBodyKind::DpBody,
                created: old.dp.is_none(),
                removed_other_body: old.animalia.is_some(),
                pose_reinitialized: false,
            })
        }
    }
}

#[cfg(test)]
mod tests;
