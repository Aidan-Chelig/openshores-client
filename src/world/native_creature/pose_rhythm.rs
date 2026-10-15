//! AuPose::Rhythm and DhDNA::Rhythm's actual float stages and timer calls.
//! Static evidence: docs/evidence/native-pose-rhythm.json and companion GNU log.
use crate::world::native_dna::NativeDhDna;
use std::io;

/// Required timer operations, not elapsed animation time or wall-clock seconds.
/// A production implementation must supply the native reference-time domain.
pub trait NativePoseRhythmTimer {
    fn start(&mut self) -> io::Result<()>;
    fn msecs_since_reference(&mut self) -> io::Result<i64>;
}

// None is the immediate-zero return, distinct from a zero frequency that still
// queries the timer and calls sinf. Full-width pose values are not truncated.
fn frequency(pose: u32) -> Option<f32> {
    match pose {
        1 | 2 | 23 | 28..=30 | 34 | 38 | 39 | 41 | 44 | 48 | 56 | 75 => Some(1.),
        4 | 17 | 22 | 45 | 136 | 137 => Some(0.5),
        24 | 53 => Some(2.5),
        25 => Some(3.),
        26 => Some(f32::from_bits(0x3e99_999a)),
        33 => Some(f32::from_bits(0x3eb3_3333)),
        37 | 47 => Some(2.),
        42 => Some(0.75),
        43 => Some(f32::from_bits(0x3f59_999a)),
        49..=52 | 108 => Some(f32::from_bits(0x3dcc_cccd)),
        0..=159 => None,
        _ => Some(0.),
    }
}

/// AuPose::Rhythm(float rate, float phase, timer). The reference-time callback
/// is invoked exactly once for a sine path and never for an immediate zero.
pub fn native_pose_rhythm(
    pose: u32,
    rate: f32,
    phase: f32,
    mut reference_msecs: impl FnMut() -> io::Result<i64>,
) -> io::Result<f32> {
    let Some(frequency) = frequency(pose) else {
        return Ok(0.);
    };
    let millis = reference_msecs()?;
    let fraction = (((millis as f64) % 60_000_f64) / 60_000_f64) as f32;
    let angle = (((fraction * 60_f32) * rate) * frequency) * f32::from_bits(0x4049_0fdb);
    Ok(((angle + angle) + phase).sin())
}

/// DhDNA::Rhythm always starts its temporary timer, even if AuPose will take an
/// immediate-zero branch. The start call's void return is not a phase value.
pub fn native_dh_pose_rhythm(
    dna: &NativeDhDna,
    pose: u32,
    phase: f32,
    speed: f32,
    timer: &mut impl NativePoseRhythmTimer,
) -> io::Result<f32> {
    let denominator = 4_f32 / (f32::from(dna.0[17] & 15) + 1_f32) + 1_f32;
    let rate = speed / denominator;
    timer.start()?;
    native_pose_rhythm(pose, rate, phase, || timer.msecs_since_reference())
}

#[cfg(test)]
mod tests;
