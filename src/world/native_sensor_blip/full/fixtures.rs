use super::*;

pub(super) const PROFILES: [FloatEncoding; 3] = [
    FloatEncoding::TypeWidths,
    FloatEncoding::SinglePrecision,
    FloatEncoding::DoublePrecision,
];

pub(super) fn float(bytes: &mut Vec<u8>, value: f64, profile: FloatEncoding) {
    if profile == FloatEncoding::DoublePrecision {
        bytes.extend(value.to_be_bytes());
    } else {
        bytes.extend((value as f32).to_be_bytes());
    }
}

pub(super) fn string(bytes: &mut Vec<u8>, value: Option<&str>) {
    match value {
        None => bytes.extend(u32::MAX.to_be_bytes()),
        Some(value) => {
            let units: Vec<_> = value.encode_utf16().collect();
            bytes.extend((units.len() as u32 * 2).to_be_bytes());
            for unit in units {
                bytes.extend(unit.to_be_bytes());
            }
        }
    }
}

pub(super) fn prefix(profile: FloatEncoding) -> Vec<u8> {
    let mut bytes = 0xfedcba98_u32.to_be_bytes().to_vec();
    for value in [1.0000000001, -0., -17.25] {
        float(&mut bytes, value, profile);
    }
    bytes
}

fn empire(bytes: &mut Vec<u8>) -> NativeBlipEmpireFields {
    bytes.extend((-123456789_i32).to_be_bytes());
    bytes.push(0xfe);
    NativeBlipEmpireFields {
        field_28: -123456789,
        field_2c: 0xfe,
    }
}

// Fixtures use the independently reviewed full-reader ledger, not decoder
// output or a shared production family table. Values distinguish adjacent
// fields, float/double overloads, signed/raw integers and null/empty strings.
pub(super) fn fixture(
    kind: u8,
    profile: FloatEncoding,
    flags: u8,
    byte: u8,
) -> (Vec<u8>, NativeBlipFullBody) {
    let mut bytes = prefix(profile);
    let body = match kind {
        1 | 4 | 8 => {
            let empire = empire(&mut bytes);
            bytes.extend([0xff, 0xa5, byte]);
            NativeBlipFullBody::Vehicle {
                empire,
                field_2e: 0x7fa5,
                field_30: 0,
                field_31: byte.max(1),
            }
        }
        2 | 3 | 5 | 6 | 11 | 12 | 27 => {
            let empire = empire(&mut bytes);
            bytes.extend([0, 0, 0, 4, 0, 0xff, 0x80, 0x7f]);
            let suffix = if kind == 2 {
                bytes.push(byte);
                NativeBlipCreatureSuffix::Animal {
                    field_40: byte != 0,
                }
            } else {
                string(&mut bytes, Some("人🚀"));
                NativeBlipCreatureSuffix::Name {
                    field_40: Some("人🚀".into()),
                }
            };
            NativeBlipFullBody::Creature(NativeBlipCreature {
                empire,
                species_30: NativeSpecies(Some(vec![0, 0xff, 0x80, 0x7f])),
                suffix,
            })
        }
        7 | 9 | 10 | 13 | 15 | 43 => {
            bytes.extend([byte, 0x81]);
            let field_2a = if [7, 9, 13].contains(&kind) {
                bytes.push(0xf3);
                Some(0xf3)
            } else {
                None
            };
            NativeBlipFullBody::World {
                field_28: byte != 0,
                field_29: 0x81,
                field_2a,
            }
        }
        14 => {
            bytes.push(byte);
            float(&mut bytes, 1.0000000001, profile);
            NativeBlipFullBody::Ring {
                field_28: byte != 0,
                field_2c: 1.,
            }
        }
        16 | 17 | 18 | 19 | 20 | 23 | 24 => {
            let empire = empire(&mut bytes);
            bytes.extend(i32::MIN.to_be_bytes());
            string(&mut bytes, None);
            bytes.extend([0, byte, 0xff]);
            bytes.extend(i32::MAX.to_be_bytes());
            string(&mut bytes, Some(""));
            float(&mut bytes, 1.0000000001, profile);
            bytes.push(byte);
            let mut point_58 = [1.0000000001, -0., 1e100_f64];
            for value in &mut point_58 {
                if profile == FloatEncoding::SinglePrecision {
                    bytes.extend((*value as f32).to_be_bytes());
                    *value = f64::from(*value as f32);
                } else {
                    bytes.extend(value.to_be_bytes());
                }
            }
            bytes.extend((-987654321_i32).to_be_bytes());
            let shield_74 = if [18, 20, 24].contains(&kind) {
                float(&mut bytes, -2.25, profile);
                Some(-2.25)
            } else {
                None
            };
            NativeBlipFullBody::Spacecraft {
                fields: NativeBlipSpacecraft {
                    empire,
                    field_30: i32::MIN,
                    field_38: None,
                    field_40: false,
                    field_41: byte != 0,
                    field_42: true,
                    field_44: i32::MAX,
                    field_48: Some(String::new()),
                    field_50: 1.,
                    field_54: byte.max(1),
                    point_58,
                    field_70: -987654321,
                },
                shield_74,
            }
        }
        21 | 22 | 28 | 40 | 41 => {
            bytes.extend([flags, byte]);
            float(&mut bytes, -3.75, profile);
            NativeBlipFullBody::Star {
                field_28: flags >> 7,
                field_2c: -3.75,
                field_30: u16::from(flags & 15) | (u16::from(byte & 15) << 8),
                field_32: byte >> 4,
            }
        }
        25 | 26 => {
            let empire = empire(&mut bytes);
            bytes.extend([0xfa, 0xfe, 0xdc]);
            string(&mut bytes, Some("🚀"));
            NativeBlipFullBody::Transporter {
                empire,
                field_2d: 0xfa,
                field_2e: 0xfedc,
                field_30: Some("🚀".into()),
            }
        }
        29 => {
            let empire = empire(&mut bytes);
            bytes.extend([0xff, 0xa5, 0xc1, 0xd2, byte]);
            NativeBlipFullBody::Missile {
                empire,
                field_2e: 0xffa5,
                field_30: 0xc1,
                field_31: 0xd2,
                field_32: byte.max(1),
            }
        }
        30 => {
            string(&mut bytes, Some(" beacon 🚀 "));
            NativeBlipFullBody::Beacon {
                field_28: Some(" beacon 🚀 ".into()),
            }
        }
        31 => NativeBlipFullBody::Anomaly,
        32..=37 | 39 => {
            let empire = empire(&mut bytes);
            bytes.push(flags);
            bytes.extend(i32::MIN.to_be_bytes());
            bytes.push(0xa5);
            bytes.extend(i32::MAX.to_be_bytes());
            let field_40 = if flags & 2 != 0 {
                string(&mut bytes, Some("基地"));
                Some("基地".into())
            } else {
                None
            };
            float(&mut bytes, -4.25, profile);
            bytes.push(byte);
            bytes.extend((-7_i32).to_be_bytes());
            NativeBlipFullBody::Bd(NativeBlipBd {
                empire,
                field_35: flags & 1,
                field_30: i32::MIN,
                field_34: 0xa5,
                field_38: i32::MAX,
                field_40,
                field_48: -4.25,
                field_4c: byte.max(1),
                field_50: -7,
            })
        }
        38 => {
            let empire = empire(&mut bytes);
            string(&mut bytes, Some(""));
            bytes.extend([0xf1, 0xe2, 0xff, 0xd3]);
            NativeBlipFullBody::TransporterBd {
                empire,
                field_30: Some(String::new()),
                field_38: 0xf1,
                field_39: 0xe2,
                field_3a: 0xffd3,
            }
        }
        42 => {
            bytes.extend([0xff, 0x81, 0xfe]);
            NativeBlipFullBody::Asteroid {
                field_28: 0xff81,
                field_2a: 0xfe,
            }
        }
        _ => panic!("unknown fixture type"),
    };
    (bytes, body)
}
