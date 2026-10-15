use super::*;

// Supplied executable .rdata1422de160..1422de1e5, corroborated against GNU
// and retained in docs/evidence/native-station-occupant-table.json.
const OCCUPANT_POSITIONS: [u8; 133] = [
    0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 0, 1, 3, 4, 0, 1, 3, 4, 0, 1, 3, 4, 0, 1, 3,
    4, 0, 1, 3, 4, 0, 1, 3, 4, 5, 0, 1, 3, 4, 0, 1, 3, 4, 0, 1, 3, 4, 0, 1, 3, 4, 0, 1, 3, 4, 0, 1,
    2, 3, 4, 0, 3,
];

pub fn native_station_occupant_position(pose_word: u32) -> u8 {
    pose_word
        .checked_sub(3)
        .and_then(|index| OCCUPANT_POSITIONS.get(index as usize))
        .copied()
        .unwrap_or(3)
}

/// Exact arguments selected by SetStationPose. This request must be applied
/// through real SitAtConsole or rejected explicitly; it is not a completed sit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use = "a station request still requires native SitAtConsole application"]
pub struct NativeStationPoseRequest {
    pub station: u16,
    pub station_type: u32,
    pub occupant_position: u8,
    pub variant: u32,
    pub trailing_station: u16,
}

impl NativeStationPoseRequest {
    /// Creature reception passes the same short twice. DpBody's station pose
    /// pointer starts three native words into the pose; Animalia uses word0.
    pub fn from_received_pose(
        kind: NativeCreatureBodyKind,
        pose: &NativeCreaturePose,
    ) -> Option<Self> {
        let word = pose.fields[if kind == NativeCreatureBodyKind::DpBody {
            3
        } else {
            0
        }];
        Self::for_pose_word(word, pose.station, pose.station)
    }

    pub fn for_pose_word(word: u32, station: u16, trailing_station: u16) -> Option<Self> {
        let (station_type, variant) = match word {
            3 => (5, 0),
            5 => (1, 0),
            6 => (2, 0),
            7 => (4, 0),
            8 => (3, 0),
            9 => (7, 0),
            10 => (8, 0),
            11 => (9, 0),
            12 => (10, 0),
            14 => (0x19, 0),
            15 => (0x18, 0),
            0x39 => (0x17, 0),
            0x3a => (0x1a, 0),
            0x3b => (0x1b, 0),
            0x3c => (0x1c, 0),
            0x3d => (0x1d, 0),
            0x3e => (0x1e, 0),
            0x3f => (0x1f, 0),
            0x40 => (0x20, 0),
            0x41 => (0, 0),
            0x42..=0x45 => (0x22, word - 0x42),
            0x46..=0x49 => (0x23, word - 0x46),
            0x54..=0x57 => (0x0b, 0),
            0x58..=0x5b => (0x0c, 0),
            0x5c..=0x5f => (0x0d, 0),
            0x60..=0x63 => (0x0e, 0),
            0x64..=0x67 => (0x0f, 0),
            0x68..=0x6b => (0x10, 0),
            0x6c..=0x70 => (0x11, 1),
            0x71..=0x74 => (0x12, 0),
            0x75..=0x78 => (0x13, 0),
            0x79..=0x7c => (0x14, 0),
            0x7d..=0x80 => (0x15, 0),
            0x81..=0x85 => (0x16, 0),
            0x86..=0x87 => (0x39, 0),
            _ => return None,
        };
        Some(Self {
            station,
            station_type,
            occupant_position: native_station_occupant_position(word),
            variant,
            trailing_station,
        })
    }
}
