//! Native RxUpdateId + leading RxUpdateBase fields; not the reference profile.
//! Input starts at the atom ID, after enclosing opcode/layer dispatch.
mod identity;
use crate::{
    atom_position::{FloatEncoding, PackedPosition},
    protocol::QtReader,
};
pub use identity::NativeIdentityPrefix;
use std::io;

/// Where decoding stops. An unknown base layer is not a type-specific body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TailKind {
    TypeSpecific,
    EffectsAndFollowing,
    SyncControllersAndFollowing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBasePrefix {
    pub id: u32,
    pub timestamp_ms: i64,
    pub flags: u8,
    pub position: Option<PackedPosition>,
    pub created_ms: Option<i64>,
    pub modified_ms: Option<i64>,
    pub in_motion: bool,
    pub tail_kind: TailKind,
    pub consumed_bytes: usize,
}

impl NativeBasePrefix {
    /// Decode only established prefix fields, preserving the unknown tail.
    /// Atomic on failure; keeps double position precision. Does not apply native
    /// ChangePos semantics, validate the enclosing opcode, or select Qt settings.
    pub fn read(input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<Self> {
        let length = input.remaining();
        let mut record = QtReader::new(input.remaining_bytes());
        let identity = NativeIdentityPrefix::read(&mut record)?;
        let id = identity.id;
        let timestamp_ms = identity.timestamp_ms;
        let flags = record.read_u8()?;
        let position = if flags & 1 != 0 {
            Some(PackedPosition::read(&mut record, encoding)?)
        } else {
            None
        };
        let created_ms = if flags & 2 != 0 {
            Some(record.read_i64()?)
        } else {
            None
        };
        let modified_ms = if flags & 4 != 0 {
            Some(record.read_i64()?)
        } else {
            None
        };
        let tail_kind = if flags & 0x10 != 0 {
            TailKind::EffectsAndFollowing
        } else if flags & 0x40 != 0 {
            TailKind::SyncControllersAndFollowing
        } else {
            TailKind::TypeSpecific
        };
        let consumed_bytes = length - record.remaining();
        input.skip(consumed_bytes)?;
        Ok(Self {
            id,
            timestamp_ms,
            flags,
            position,
            created_ms,
            modified_ms,
            in_motion: flags & 0x20 != 0,
            tail_kind,
            consumed_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_position_precedes_times_and_unknown_tail_is_not_a_body() {
        for flags in 0..=u8::MAX {
            let mut bytes = Vec::new();
            bytes.extend(0xf1234567_u32.to_be_bytes());
            bytes.extend((-123_i64).to_be_bytes());
            bytes.push(flags);
            if flags & 1 != 0 {
                bytes.push(0x83); // parent + double X position
                bytes.extend(77_u32.to_be_bytes());
                bytes.extend(12345678.125_f64.to_be_bytes());
            }
            if flags & 2 != 0 {
                bytes.extend(111_i64.to_be_bytes());
            }
            if flags & 4 != 0 {
                bytes.extend(222_i64.to_be_bytes());
            }
            let prefix_length = bytes.len();
            bytes.extend([0xde, 0xad, 0xbe, 0xef]);
            let mut input = QtReader::new(&bytes);
            let prefix = NativeBasePrefix::read(&mut input, FloatEncoding::TypeWidths).unwrap();
            assert_eq!(prefix.id, 0xf1234567);
            assert_eq!(prefix.timestamp_ms, -123);
            assert_eq!(prefix.flags, flags);
            assert_eq!(prefix.created_ms, (flags & 2 != 0).then_some(111));
            assert_eq!(prefix.modified_ms, (flags & 4 != 0).then_some(222));
            assert_eq!(prefix.in_motion, flags & 0x20 != 0);
            assert_eq!(prefix.consumed_bytes, prefix_length);
            assert_eq!(prefix.position.is_some(), flags & 1 != 0);
            assert_eq!(input.read_u32().unwrap(), 0xdeadbeef);
            if let Some(position) = prefix.position {
                assert_eq!(position.parent, Some(77));
                assert_eq!(position.position, [12345678.125, 0.0, 0.0]);
            }
            assert_eq!(
                prefix.tail_kind,
                if flags & 0x10 != 0 {
                    TailKind::EffectsAndFollowing
                } else if flags & 0x40 != 0 {
                    TailKind::SyncControllersAndFollowing
                } else {
                    TailKind::TypeSpecific
                }
            );
            for length in 0..prefix_length {
                let mut input = QtReader::new(&bytes[..length]);
                assert!(NativeBasePrefix::read(&mut input, FloatEncoding::TypeWidths).is_err());
                assert_eq!(input.remaining(), length);
            }
        }
    }
}
