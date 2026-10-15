use super::*;

/// AuStation::OpPose. Unknown station/occupant/index combinations return the
/// native standing24 fallback; they do not invent a console. Random types
/// require the real shared u32 state at AuGlobal+f4. No guessed seed is supplied.
/// This computes a pose word, not SitAtConsole lifecycle or animation effects.
pub fn native_station_operator_pose(
    station_type: u32,
    occupant_position: u32,
    variant: i32,
    random_state: Option<&mut u32>,
) -> io::Result<u32> {
    let by_occupant = match station_type {
        0x0b => Some([0x54, 0x55, 0x54, 0x56, 0x57, 0x54]),
        0x0c => Some([0x58, 0x59, 0x58, 0x5a, 0x5b, 0x58]),
        0x0d => Some([0x5c, 0x5d, 0x5c, 0x5e, 0x5f, 0x5c]),
        0x0e => Some([0x60, 0x61, 0x60, 0x62, 0x63, 0x60]),
        0x0f => Some([0x64, 0x65, 0x64, 0x66, 0x67, 0x64]),
        0x10 => Some([0x68, 0x69, 0x68, 0x6a, 0x6b, 0x68]),
        0x11 => Some([0x6d, 0x6e, 0x6d, 0x6f, 0x70, 0x6c]),
        0x12 => Some([0x71, 0x72, 0x71, 0x73, 0x74, 0x71]),
        0x13 => Some([0x75, 0x76, 0x75, 0x77, 0x78, 0x75]),
        0x14 => Some([0x79, 0x7a, 0x79, 0x7b, 0x7c, 0x79]),
        0x15 => Some([0x7d, 0x7e, 0x7d, 0x7f, 0x80, 0x7d]),
        0x16 => Some([0x81, 0x82, 0x83, 0x84, 0x85, 0x81]),
        0x39..=0x3b => Some([0x86, 0x86, 0x86, 0x87, 0x87, 0x86]),
        _ => None,
    };
    if let Some(poses) = by_occupant {
        return Ok(poses
            .get(occupant_position as usize)
            .copied()
            .unwrap_or(0x24));
    }
    Ok(match station_type {
        0 => 0x41,
        1 => 5,
        2 => 6,
        3 => 8,
        4 => 7,
        5 => 3,
        6 => 0x0d,
        7 => 9,
        8 => 0x0a,
        9 => 0x0b,
        10 => 0x0c,
        0x17 => 0x39,
        0x18 => 0x0f,
        0x19 => 0x0e,
        0x1a => 0x3a,
        0x1b => 0x3b,
        0x1c => 0x3c,
        0x1d => 0x3d,
        0x1e => 0x3e,
        0x1f => 0x3f,
        0x20 => 0x40,
        0x21 | 0x22 if (0..=3).contains(&variant) => 0x42 + variant as u32,
        0x23 if (0..=3).contains(&variant) => 0x46 + variant as u32,
        0x24..=0x38 => {
            let state = random_state.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native OpPose requires shared AuGlobal random state",
                )
            })?;
            *state = state.wrapping_mul(0x41c64e6d).wrapping_add(0x3039);
            // Native f64(seed)*4.0*2^-32 then truncation is exactly seed>>30
            // for all u32 states. Both power-of-two constants are retained.
            (if variant == 0 { 0x4c } else { 0x50 }) + (*state >> 30)
        }
        _ => 0x24,
    })
}
