use crate::{atom_position::FloatEncoding, protocol::QtReader};
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeStationGame {
    pub field_14: u8,
    pub field_16: u16,
}

impl NativeStationGame {
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let flags = record.read_u8()?;
        let field_14 = if flags & 1 != 0 {
            record.read_u8()?
        } else {
            0x22
        };
        let field_16 = if flags & 2 != 0 {
            record.read_u16()?
        } else {
            0
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(Self { field_14, field_16 })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeStationShield {
    pub fields_14: [f32; 6],
    pub field_30: f64,
}

impl NativeStationShield {
    pub fn read(input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let mut fields_14 = [0.; 6];
        for field in &mut fields_14 {
            *field = match encoding {
                FloatEncoding::DoublePrecision => record.read_f64()? as f32,
                FloatEncoding::TypeWidths | FloatEncoding::SinglePrecision => record.read_f32()?,
            };
        }
        let field_30 = match encoding {
            FloatEncoding::SinglePrecision => f64::from(record.read_f32()?),
            FloatEncoding::TypeWidths | FloatEncoding::DoublePrecision => record.read_f64()?,
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(Self {
            fields_14,
            field_30,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_game_flag_combination_resets_omitted_fields_and_stops_at_suffix() {
        for flags in 0..=255 {
            let mut bytes = vec![flags];
            if flags & 1 != 0 {
                bytes.push(0xfe);
            }
            if flags & 2 != 0 {
                bytes.extend(0x80ff_u16.to_be_bytes());
            }
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            assert_eq!(
                NativeStationGame::read(&mut input).unwrap(),
                NativeStationGame {
                    field_14: if flags & 1 != 0 { 0xfe } else { 0x22 },
                    field_16: if flags & 2 != 0 { 0x80ff } else { 0 }
                }
            );
            assert_eq!(input.read_u8().unwrap(), 0xde);
            for end in 0..length {
                let mut short = QtReader::new(&bytes[..end]);
                assert!(NativeStationGame::read(&mut short).is_err());
                assert_eq!(short.remaining(), end);
            }
        }
    }

    #[test]
    fn shield_uses_six_float_overloads_then_double_with_explicit_precision() {
        for encoding in [
            FloatEncoding::TypeWidths,
            FloatEncoding::SinglePrecision,
            FloatEncoding::DoublePrecision,
        ] {
            let values = [
                1.0000000001_f64,
                -0.,
                f64::MAX,
                f64::NAN,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ];
            let mut bytes = Vec::new();
            for value in values {
                if encoding == FloatEncoding::DoublePrecision {
                    bytes.extend(value.to_be_bytes());
                } else {
                    bytes.extend((value as f32).to_be_bytes());
                }
            }
            let double = 1.0000000001_f64;
            if encoding == FloatEncoding::SinglePrecision {
                bytes.extend((double as f32).to_be_bytes());
            } else {
                bytes.extend(double.to_be_bytes());
            }
            let length = bytes.len();
            bytes.push(0xde);
            let mut input = QtReader::new(&bytes);
            let value = NativeStationShield::read(&mut input, encoding).unwrap();
            for (actual, expected) in value
                .fields_14
                .into_iter()
                .zip(values.map(|value| value as f32))
            {
                if expected.is_nan() {
                    assert!(actual.is_nan());
                } else {
                    assert_eq!(actual.to_bits(), expected.to_bits());
                }
            }
            assert_eq!(
                value.field_30.to_bits(),
                if encoding == FloatEncoding::SinglePrecision {
                    f64::from(double as f32).to_bits()
                } else {
                    double.to_bits()
                }
            );
            assert_eq!(input.read_u8().unwrap(), 0xde);
            for end in 0..length {
                let mut short = QtReader::new(&bytes[..end]);
                assert!(NativeStationShield::read(&mut short, encoding).is_err());
                assert_eq!(short.remaining(), end);
            }
        }
    }
}
