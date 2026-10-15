use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeDpHealthFlags {
    pub byte_19: u8,
    pub byte_1a: u8,
    pub byte_1d: u8,
}

pub trait NativePartVolumeContext {
    /// Stable actual model-part collection for this exclusive Rust operation.
    fn part_count(&mut self) -> io::Result<usize>;
    /// Actual part virtual+98. Unknown model geometry must return Unsupported.
    fn part_volume(&mut self, index: usize) -> io::Result<f32>;
}

/// DpBody constructor writes float68=0. Model mutation must separately perform
/// its verified cache invalidation; this helper never guesses when to reset it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeDpHealthState {
    pub cached_volume_68: f32,
}
impl NativeDpHealthState {
    pub fn max_hit_points(
        &mut self,
        flags: NativeDpHealthFlags,
        context: &mut impl NativePartVolumeContext,
    ) -> io::Result<NativeHealthResult> {
        if self.cached_volume_68 == 0. {
            let count = bounded_parts(context.part_count()?)?;
            for index in 0..count {
                self.cached_volume_68 += context.part_volume(index)?;
            }
        }
        let (mut value, replaced) = volume_hp(self.cached_volume_68, 0x41ec_86eb);
        if flags.byte_1d & 4 != 0 {
            if flags.byte_19 & 3 == 0 || flags.byte_1a & 6 == 0 {
                value = (u32::from(value) * 2 / 3) as u16;
            } else {
                value >>= 1;
            }
        }
        Ok(finish(value, replaced))
    }
}
