//! Channel-specific gameplay commands accepted by the supplied server, not UI bindings.
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatCommand {
    /// Top-level gear only. Nested source-path encoding is not inferred.
    DropGear {
        source_slot: u8,
        source_subindex: u8,
    },
}

impl ChatCommand {
    pub fn encode(self) -> io::Result<Vec<u8>> {
        match self {
            Self::DropGear {
                source_slot,
                source_subindex,
            } => {
                if !(1..=9).contains(&source_slot) || source_subindex > 15 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "drop requires slot 1..=9 and subindex 0..=15",
                    ));
                }
                Ok(vec![0x57, source_slot, source_subindex])
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SceneCommand {
    /// Raw ten-byte pose state; zero auxiliary fields avoid weapon triggers.
    /// Pose values are not assigned animation or keyboard meanings here.
    PoseReport { poses: [u8; 10] },
    /// Raw client-reported coordinates, not a movement intent or camera mapping.
    PositionReport {
        projected: [f64; 3],
        rotation: [f64; 3],
    },
    /// Minimal delta-tick target pin, not a direct pickup opcode or movement.
    TargetPin { target: u32 },
    /// Zero clears the target. Outbound 0x18 is not an inbound scene manifest.
    LookAt { target: u32 },
    /// Destination subindex is assigned by the server, not chosen by the client.
    MoveGear {
        source_slot: u8,
        source_subindex: u8,
        destination_slot: u8,
    },
}

impl SceneCommand {
    pub fn encode(self) -> io::Result<Vec<u8>> {
        match self {
            Self::PoseReport { poses } => {
                let mut bytes = vec![0x42, 0x00, 0x08];
                bytes.extend(poses);
                bytes.extend([0; 6]);
                Ok(bytes)
            }
            Self::PositionReport {
                projected,
                rotation,
            } => {
                // The source broadcasts these doubles to peers as six f32s.
                // Reject values that would overflow that downstream conversion.
                if projected
                    .iter()
                    .chain(rotation.iter())
                    .any(|value| !value.is_finite() || value.abs() > f64::from(f32::MAX))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "position report values must be finite and fit in f32",
                    ));
                }
                let mut bytes = vec![0x42, 0x00, 0x02];
                for value in projected.into_iter().chain(rotation) {
                    bytes.extend(value.to_be_bytes());
                }
                Ok(bytes)
            }
            Self::TargetPin { target } => {
                let mut bytes = vec![0x42, 0x04, 0x00];
                bytes.extend(target.to_be_bytes());
                Ok(bytes)
            }
            Self::LookAt { target } => {
                let mut bytes = vec![0x18];
                bytes.extend(target.to_be_bytes());
                Ok(bytes)
            }
            Self::MoveGear {
                source_slot,
                source_subindex,
                destination_slot,
            } => {
                if [source_slot, source_subindex, destination_slot]
                    .iter()
                    .any(|n| *n > 15)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "gear slot/subindex must be in 0..=15",
                    ));
                }
                // The reference handler reads but does not interpret this flag.
                // Zero is our explicit local test profile, not original-client parity.
                Ok(vec![
                    0x60,
                    source_slot,
                    source_subindex,
                    0,
                    destination_slot,
                ])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pose_report_preserves_all_ten_bytes_and_neutral_auxiliary_fields() {
        let poses = [0, 1, 2, 3, 4, 5, 6, 7, 0x73, 0xff];
        let bytes = SceneCommand::PoseReport { poses }.encode().unwrap();
        assert_eq!(bytes.len(), 19);
        assert_eq!(&bytes[..3], &[0x42, 0, 8]);
        assert_eq!(&bytes[3..13], &poses);
        assert_eq!(&bytes[13..], &[0; 6]);
    }
    #[test]
    fn position_report_uses_big_endian_doubles_and_only_flag_two() {
        let values = [100.5, 200.25, -300.125, 0.25, -0.5, 0.75];
        let bytes = SceneCommand::PositionReport {
            projected: [values[0], values[1], values[2]],
            rotation: [values[3], values[4], values[5]],
        }
        .encode()
        .unwrap();
        assert_eq!(bytes.len(), 51);
        assert_eq!(&bytes[..3], &[0x42, 0, 2]);
        for (chunk, expected) in bytes[3..].chunks_exact(8).zip(values) {
            assert_eq!(f64::from_be_bytes(chunk.try_into().unwrap()), expected);
        }
        for invalid in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            -f64::MAX,
        ] {
            for rotation_invalid in [false, true] {
                assert!(
                    SceneCommand::PositionReport {
                        projected: if rotation_invalid {
                            [0.0; 3]
                        } else {
                            [invalid; 3]
                        },
                        rotation: if rotation_invalid {
                            [invalid; 3]
                        } else {
                            [0.0; 3]
                        },
                    }
                    .encode()
                    .is_err()
                );
            }
        }
    }
    #[test]
    fn source_layouts_and_slot_bounds_are_explicit() {
        assert_eq!(
            SceneCommand::TargetPin { target: 0x70000000 }
                .encode()
                .unwrap(),
            [0x42, 4, 0, 0x70, 0, 0, 0]
        );
        assert_eq!(
            ChatCommand::DropGear {
                source_slot: 5,
                source_subindex: 0
            }
            .encode()
            .unwrap(),
            [0x57, 5, 0]
        );
        for (source_slot, source_subindex) in [(0, 0), (10, 0), (5, 16)] {
            assert!(
                ChatCommand::DropGear {
                    source_slot,
                    source_subindex
                }
                .encode()
                .is_err()
            );
        }
        assert_eq!(
            SceneCommand::LookAt { target: 0x89abcdef }
                .encode()
                .unwrap(),
            [0x18, 0x89, 0xab, 0xcd, 0xef]
        );
        assert_eq!(
            SceneCommand::LookAt { target: 0 }.encode().unwrap(),
            [0x18, 0, 0, 0, 0]
        );
        assert_eq!(
            SceneCommand::MoveGear {
                source_slot: 5,
                source_subindex: 2,
                destination_slot: 1
            }
            .encode()
            .unwrap(),
            [0x60, 5, 2, 0, 1]
        );
        for (source_slot, source_subindex, destination_slot) in
            [(16, 0, 0), (0, 16, 0), (0, 0, 255)]
        {
            assert!(
                SceneCommand::MoveGear {
                    source_slot,
                    source_subindex,
                    destination_slot
                }
                .encode()
                .is_err()
            );
        }
    }
}
