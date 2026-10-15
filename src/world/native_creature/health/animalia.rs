use super::*;

pub trait NativeAnimaliaHealthContext {
    /// Actual selected chromatid's Volume, not a guessed body/mesh bounding box.
    fn chromatid_volume(&mut self) -> io::Result<f32>;
    fn part_count(&mut self) -> io::Result<usize>;
    /// False for null/non-wing parts; actual AuAnimalWing::CanFly for wings.
    /// Unresolved part identity or capability must fail explicitly.
    fn part_is_flying_wing(&mut self, index: usize) -> io::Result<bool>;
}
pub fn native_animalia_max_hit_points(
    context: &mut impl NativeAnimaliaHealthContext,
) -> io::Result<NativeHealthResult> {
    let (mut value, replaced) = volume_hp(context.chromatid_volume()?, 0x421c_f478);
    let count = bounded_parts(context.part_count()?)?;
    for index in 0..count {
        if context.part_is_flying_wing(index)? {
            value >>= 1;
            break;
        }
    }
    Ok(finish(value, replaced))
}
