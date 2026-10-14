//! Native RxUpdateId fields, before base flags/body. No stale-time policy.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeIdentityPrefix {
    pub id: u32,
    pub timestamp_ms: i64,
}
impl NativeIdentityPrefix {
    /// Read the established 12-byte identity stage atomically. Native writes ID
    /// then time directly; rejecting incomplete Rust input before mutation is a
    /// safety boundary, not Qt stream-error or full packet-dispatch parity.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let result = Self {
            id: record.read_u32()?,
            timestamp_ms: record.read_i64()?,
        };
        input.skip(12)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_preserves_raw_widths_and_stops_before_flags() {
        for id in [0, 1, 0x80000000, u32::MAX] {
            for timestamp_ms in [i64::MIN, -1, 0, i64::MAX] {
                let mut bytes = id.to_be_bytes().to_vec();
                bytes.extend(timestamp_ms.to_be_bytes());
                bytes.extend([0xff, 0xde, 0xad]);
                let mut input = QtReader::new(&bytes);
                assert_eq!(
                    NativeIdentityPrefix::read(&mut input).unwrap(),
                    NativeIdentityPrefix { id, timestamp_ms }
                );
                assert_eq!(input.remaining_bytes(), [0xff, 0xde, 0xad]);
                for length in 0..12 {
                    let mut input = QtReader::new(&bytes[..length]);
                    assert!(NativeIdentityPrefix::read(&mut input).is_err());
                    assert_eq!(input.remaining(), length);
                }
            }
        }
    }
}
