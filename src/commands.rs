//! Outbound scene commands accepted by the supplied server, not UI bindings.
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneCommand {
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
    fn source_layouts_and_slot_bounds_are_explicit() {
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
