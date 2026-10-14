//! Bounded source-profile scene-capture replay.
use super::{WorldState, invalid};
use crate::protocol::read_frame;
use std::io::{self, BufRead};

/// Replay scene-only captures with the same bounds as the local network probe.
pub fn read_capture(reader: &mut impl BufRead) -> io::Result<WorldState> {
    let mut world = WorldState::default();
    let mut count = 0;
    let mut bytes = 0usize;
    while !reader.fill_buf()?.is_empty() {
        count += 1;
        if count > 20_000 {
            return Err(invalid("capture exceeds packet budget"));
        }
        let packet = read_frame(reader, 16 * 1024 * 1024)?;
        bytes += packet.len();
        if bytes > 128 * 1024 * 1024 {
            return Err(invalid("capture exceeds byte budget"));
        }
        world.ingest(&packet)?;
    }
    Ok(world)
}
