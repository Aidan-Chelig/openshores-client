use crate::{
    atom_position::FloatEncoding, protocol::QtReader, world::native_condition::NativeConditionList,
};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeStationSurgery {
    pub conditions_48: NativeConditionList,
    pub field_38: u16,
    pub points_40: Vec<[i32; 2]>,
    pub field_14: u16,
    pub field_16: u8,
    pub field_18: u16,
    pub field_20: Option<String>,
    pub field_28: u16,
}

impl NativeStationSurgery {
    /// Fresh value in the verified Qt7 layout. Atomic errors and a 20,000-point
    /// budget are Rust safety policies, not native allocation/status behavior.
    pub fn read_qt7(input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let mut conditions_48 = NativeConditionList::default();
        conditions_48.receive(&mut record, encoding)?;
        let field_38 = record.read_u16()?;
        let count = record.read_u32()?;
        if count > 20_000 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native surgery point budget exceeded",
            ));
        }
        let mut points_40 = Vec::new();
        for _ in 0..count {
            points_40.push([record.read_i32()?, record.read_i32()?]);
        }
        let value = Self {
            conditions_48,
            field_38,
            points_40,
            field_14: record.read_u16()?,
            field_16: record.read_u8()?,
            field_18: record.read_u16()?,
            field_20: record.read_string()?,
            field_28: record.read_u16()?,
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{protocol::QtWriter, world::native_condition::NativeConditionBody};

    #[test]
    fn surgery_receives_real_conditions_signed_points_and_trailing_fields() {
        for text in [None, Some(""), Some("surgery 🦀")] {
            let mut bytes = 2_u16.to_be_bytes().to_vec();
            bytes.push(9);
            bytes.extend(i64::MIN.to_be_bytes());
            bytes.extend([0xa3, 0]); // null Pregnant, raw zero
            let mut writer = QtWriter::default();
            writer.write_string(None).unwrap();
            bytes.extend(writer.into_bytes());
            bytes.extend(u32::MAX.to_be_bytes());
            bytes.push(8);
            bytes.extend(17_i64.to_be_bytes()); // following Fire
            bytes.extend(0x80fe_u16.to_be_bytes());
            bytes.extend(2_u32.to_be_bytes());
            for value in [i32::MIN, i32::MAX, -1, 0] {
                bytes.extend(value.to_be_bytes());
            }
            bytes.extend(0xffff_u16.to_be_bytes());
            bytes.push(0xfe);
            bytes.extend(0x8000_u16.to_be_bytes());
            let mut writer = QtWriter::default();
            writer.write_string(text).unwrap();
            bytes.extend(writer.into_bytes());
            bytes.extend(0xabcd_u16.to_be_bytes());
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let value =
                NativeStationSurgery::read_qt7(&mut input, FloatEncoding::TypeWidths).unwrap();
            assert_eq!(value.field_38, 0x80fe);
            assert_eq!(value.points_40, vec![[i32::MIN, i32::MAX], [-1, 0]]);
            assert_eq!(
                (
                    value.field_14,
                    value.field_16,
                    value.field_18,
                    value.field_28
                ),
                (0xffff, 0xfe, 0x8000, 0xabcd)
            );
            assert_eq!(value.field_20.as_deref(), text);
            assert_eq!(value.conditions_48.0.len(), 2);
            assert!(matches!(
                value.conditions_48.0[0].body,
                NativeConditionBody::Pregnant {
                    field_10: None,
                    field_18: 0,
                    id_1c: u32::MAX,
                    field_20: 3,
                    field_28: None
                }
            ));
            assert_eq!(value.conditions_48.0[1].body, NativeConditionBody::Fire);
            assert_eq!(input.read_u8().unwrap(), 0xde);
            for end in 0..length {
                let mut short = QtReader::new(&bytes[..end]);
                assert!(
                    NativeStationSurgery::read_qt7(&mut short, FloatEncoding::TypeWidths).is_err()
                );
                assert_eq!(short.remaining(), end);
            }
        }
    }

    #[test]
    fn point_budget_is_bounded_before_allocation_and_empty_lists_have_no_payload() {
        for count in [0_u32, 20_000, 20_001, 0x80000000, u32::MAX] {
            let mut bytes = vec![0, 0, 0, 0];
            bytes.extend(count.to_be_bytes());
            if count <= 20_000 {
                bytes.extend(std::iter::repeat_n(0, count as usize * 8));
                bytes.extend([0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0, 0]);
            }
            let mut input = QtReader::new(&bytes);
            if count > 20_000 {
                assert_eq!(
                    NativeStationSurgery::read_qt7(&mut input, FloatEncoding::TypeWidths)
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::InvalidData
                );
                assert_eq!(input.remaining(), bytes.len());
            } else {
                let value =
                    NativeStationSurgery::read_qt7(&mut input, FloatEncoding::TypeWidths).unwrap();
                assert_eq!(value.points_40.len(), count as usize);
                assert!(value.conditions_48.0.is_empty());
                assert_eq!(input.remaining(), 0);
            }
        }
    }
}
