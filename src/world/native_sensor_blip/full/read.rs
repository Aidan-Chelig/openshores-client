use super::*;

impl NativeSensorBlipFull {
    /// After the discriminator, read a fresh factory-bound +158 body. Unknown
    /// types return None, never a guessed skip. Atomic errors and bounded raw
    /// species/strict QString handling are Rust policies, not Qt error parity.
    pub fn read(
        kind: u8,
        input: &mut QtReader<'_>,
        encoding: FloatEncoding,
    ) -> io::Result<Option<Self>> {
        if !(1..=43).contains(&kind) {
            return Ok(None);
        }
        let mut record = QtReader::new(input.remaining_bytes());
        let (id_08, position_10) = read_prefix(&mut record, encoding)?;
        let body = match kind {
            31 => NativeBlipFullBody::Anomaly,
            30 => NativeBlipFullBody::Beacon {
                field_28: record.read_string()?,
            },
            14 => NativeBlipFullBody::Ring {
                field_28: boolean(&mut record)?,
                field_2c: read_float(&mut record, encoding)?,
            },
            7 | 9 | 10 | 13 | 15 | 43 => NativeBlipFullBody::World {
                field_28: boolean(&mut record)?,
                field_29: record.read_u8()?,
                field_2a: if matches!(kind, 7 | 9 | 13) {
                    Some(record.read_u8()?)
                } else {
                    None
                },
            },
            42 => NativeBlipFullBody::Asteroid {
                field_28: record.read_u16()?,
                field_2a: record.read_u8()?,
            },
            2 | 3 | 5 | 6 | 11 | 12 | 27 => {
                let empire = empire(&mut record)?;
                let species_30 = NativeSpecies::read(&mut record)?;
                let suffix = if kind == 2 {
                    NativeBlipCreatureSuffix::Animal {
                        field_40: boolean(&mut record)?,
                    }
                } else {
                    NativeBlipCreatureSuffix::Name {
                        field_40: record.read_string()?,
                    }
                };
                NativeBlipFullBody::Creature(NativeBlipCreature {
                    empire,
                    species_30,
                    suffix,
                })
            }
            21 | 22 | 28 | 40 | 41 => {
                let a = record.read_u8()?;
                let b = record.read_u8()?;
                let field_2c = read_float(&mut record, encoding)?;
                NativeBlipFullBody::Star {
                    field_28: a >> 7,
                    field_2c,
                    field_30: ((u16::from(b) << 8) | u16::from(a)) & 0x0f0f,
                    field_32: b >> 4,
                }
            }
            1 | 4 | 8 => {
                let empire = empire(&mut record)?;
                let field_2e = record.read_u16()? & 0x7fff;
                let field_31 = record.read_u8()?.max(1);
                NativeBlipFullBody::Vehicle {
                    empire,
                    field_2e,
                    field_30: 0,
                    field_31,
                }
            }
            29 => NativeBlipFullBody::Missile {
                empire: empire(&mut record)?,
                field_2e: record.read_u16()?,
                field_30: record.read_u8()?,
                field_31: record.read_u8()?,
                field_32: record.read_u8()?.max(1),
            },
            25 | 26 => NativeBlipFullBody::Transporter {
                empire: empire(&mut record)?,
                field_2d: record.read_u8()?,
                field_2e: record.read_u16()?,
                field_30: record.read_string()?,
            },
            38 => NativeBlipFullBody::TransporterBd {
                empire: empire(&mut record)?,
                field_30: record.read_string()?,
                field_38: record.read_u8()?,
                field_39: record.read_u8()?,
                field_3a: record.read_u16()?,
            },
            32..=37 | 39 => NativeBlipFullBody::Bd(bd(&mut record, encoding)?),
            16..=20 | 23 | 24 => {
                let fields = spacecraft(&mut record, encoding)?;
                let shield_74 = if matches!(kind, 18 | 20 | 24) {
                    Some(read_float(&mut record, encoding)?)
                } else {
                    None
                };
                NativeBlipFullBody::Spacecraft { fields, shield_74 }
            }
            _ => unreachable!("all native factory types covered"),
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(Some(Self {
            kind,
            id_08,
            position_10,
            body,
        }))
    }
}

fn boolean(input: &mut QtReader<'_>) -> io::Result<bool> {
    Ok(input.read_u8()? != 0)
}
fn empire(input: &mut QtReader<'_>) -> io::Result<NativeBlipEmpireFields> {
    Ok(NativeBlipEmpireFields {
        field_28: input.read_i32()?,
        field_2c: input.read_u8()?,
    })
}
fn bd(input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<NativeBlipBd> {
    let empire = empire(input)?;
    let flags = input.read_u8()?;
    let field_30 = input.read_i32()?;
    let field_34 = input.read_u8()?;
    let field_38 = input.read_i32()?;
    // Full blip reception constructs a fresh factory object. Omitted string
    // retains its shared-null factory default, not a reused object's old name.
    let field_40 = if flags & 2 != 0 {
        input.read_string()?
    } else {
        None
    };
    Ok(NativeBlipBd {
        empire,
        field_35: flags & 1,
        field_30,
        field_34,
        field_38,
        field_40,
        field_48: read_float(input, encoding)?,
        field_4c: input.read_u8()?.max(1),
        field_50: input.read_i32()?,
    })
}
fn spacecraft(
    input: &mut QtReader<'_>,
    encoding: FloatEncoding,
) -> io::Result<NativeBlipSpacecraft> {
    let empire = empire(input)?;
    let field_30 = input.read_i32()?;
    let field_38 = input.read_string()?;
    let field_40 = boolean(input)?;
    let field_41 = boolean(input)?;
    let field_42 = boolean(input)?;
    let field_44 = input.read_i32()?;
    let field_48 = input.read_string()?;
    let field_50 = read_float(input, encoding)?;
    let field_54 = input.read_u8()?.max(1);
    let mut point_58 = [0.; 3];
    for value in &mut point_58 {
        *value = match encoding {
            FloatEncoding::SinglePrecision => f64::from(input.read_f32()?),
            FloatEncoding::TypeWidths | FloatEncoding::DoublePrecision => input.read_f64()?,
        };
    }
    let field_70 = input.read_i32()?;
    Ok(NativeBlipSpacecraft {
        empire,
        field_30,
        field_38,
        field_40,
        field_41,
        field_42,
        field_44,
        field_48,
        field_50,
        field_54,
        point_58,
        field_70,
    })
}
