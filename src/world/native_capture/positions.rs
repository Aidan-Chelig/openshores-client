use super::{CaptureLimits, NativeCapture};
use crate::world::{
    native_atom::NativeAtomPrefix,
    native_position_updates::{DestinationContext, NativePositionReceiver, NativePositionUpdate},
    native_subatoms::ReparentRegistrationContext,
};
use std::{fmt, io, time::Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct ReplayedNativePosition {
    pub frame: usize,
    pub id: u32,
    pub outcome: NativePositionUpdate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativePositionReplay {
    /// Includes unresolved obligations, not just successfully applied changes.
    pub positions: Vec<ReplayedNativePosition>,
    pub non_atom_frames: usize,
    pub no_position_frames: usize,
}

#[derive(Debug)]
pub struct NativePositionReplayError {
    /// None is preflight capture-budget rejection; Some is a failed frame.
    pub frame: Option<usize>,
    pub completed: NativePositionReplay,
    pub error: io::Error,
}
impl fmt::Display for NativePositionReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.frame {
            Some(frame) => write!(
                f,
                "native position replay failed at frame {frame}: {}",
                self.error
            ),
            None => write!(f, "native position replay rejected capture: {}", self.error),
        }
    }
}
impl std::error::Error for NativePositionReplayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl NativeCapture {
    /// Replay only existing-object base positions in capture order. Caller
    /// supplies destination observations and monotonic time for each position;
    /// recorded timestamps are not silently converted to local clock epochs.
    /// Bodies/transport remain opaque. Input frames and payloads stay unchanged.
    /// Stop on failure, retaining earlier effects and outcomes. A failing frame
    /// can retain partial effects under the receiver's callback-error policy.
    pub fn replay_positions<O: ReparentRegistrationContext>(
        &self,
        receiver: &mut NativePositionReceiver<'_, O>,
        mut context: impl FnMut(usize, &NativeAtomPrefix<'_>) -> (DestinationContext, Instant),
    ) -> Result<NativePositionReplay, NativePositionReplayError> {
        let hard = CaptureLimits::default();
        let mut total = 0usize;
        if self.frames.len() > hard.frames
            || self.frames.iter().any(|frame| {
                let size = frame.payload().len();
                total = total.saturating_add(size);
                size > hard.frame_bytes || total > hard.payload_bytes
            })
        {
            return Err(NativePositionReplayError {
                frame: None,
                completed: NativePositionReplay::default(),
                error: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "native position replay exceeds capture budget",
                ),
            });
        }
        let mut replay = NativePositionReplay::default();
        for (index, frame) in self.frames.iter().enumerate() {
            let Some((kind, base)) = frame.prefix() else {
                replay.non_atom_frames += 1;
                continue;
            };
            if base.position.is_none() {
                replay.no_position_frames += 1;
                continue;
            }
            let prefix = NativeAtomPrefix {
                kind: *kind,
                base: base.clone(),
                remainder: frame.remainder(),
            };
            let (destination, now) = context(index, &prefix);
            match receiver.apply_at(&prefix, destination, now) {
                Ok(Some(outcome)) => replay.positions.push(ReplayedNativePosition {
                    frame: index,
                    id: base.id,
                    outcome,
                }),
                Ok(None) => unreachable!("retained prefix has a position"),
                Err(error) => {
                    return Err(NativePositionReplayError {
                        frame: Some(index),
                        completed: replay,
                        error,
                    });
                }
            }
        }
        Ok(replay)
    }
}

#[cfg(test)]
#[path = "position_tests/mod.rs"]
mod tests;
