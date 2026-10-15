use crate::{atom_position::FloatEncoding, protocol::QtReader};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeWaypoint {
    pub ids: [u32; 2],
    pub position: [f64; 3],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeWaypointRoute(pub Vec<NativeWaypoint>);

impl NativeWaypointRoute {
    /// Count precedes clearing; completed appends survive later Rust errors.
    /// All 16 count bits are used. No allocation is made from count alone.
    pub fn receive(&mut self, input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<()> {
        let count = input.read_u16()?;
        self.0.clear();
        for _ in 0..count {
            let ids = [input.read_u32()?, input.read_u32()?];
            let mut position = [0.; 3];
            for value in &mut position {
                *value = match encoding {
                    FloatEncoding::SinglePrecision => f64::from(input.read_f32()?),
                    FloatEncoding::TypeWidths | FloatEncoding::DoublePrecision => {
                        input.read_f64()?
                    }
                };
            }
            self.0.push(NativeWaypoint { ids, position });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeStationNav {
    pub field_14: u32,
    pub field_18: u8,
    pub field_1c: u32,
    pub field_64: u32,
    pub field_20: u32,
    pub vector_28: [f64; 3],
    pub vector_40: [f64; 3],
    pub id_68: u32,
    pub id_58: u32,
    pub field_6c: u8,
    pub field_60: u32,
    pub vector_70: [f64; 3],
    pub field_5c: u8,
    pub route_90: NativeWaypointRoute,
}

impl NativeStationNav {
    /// Fresh-value atomicity is Rust safety, not native partial-write parity.
    pub fn read(input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let flags = record.read_u8()?;
        let field_1c = u32::from(record.read_u8()?);
        let field_64 = u32::from(record.read_u8()?);
        let field_20 = u32::from(record.read_u8()?);
        let vector_28 = vector(&mut record, encoding)?;
        let vector_40 = vector(&mut record, encoding)?;
        let id_68 = record.read_u32()?;
        let id_58 = record.read_u32()?;
        let field_6c = record.read_u8()?;
        let field_60 = u32::from(record.read_u8()?);
        let vector_70 = vector(&mut record, encoding)?;
        let mut route_90 = NativeWaypointRoute::default();
        route_90.receive(&mut record, encoding)?;
        let value = Self {
            field_14: u32::from((flags >> 6) & 1)
                | (u32::from(flags & 1) << 8)
                | (u32::from((flags >> 1) & 1) << 16)
                | (u32::from((flags >> 2) & 1) << 24),
            field_18: (flags >> 3) & 1,
            field_5c: (flags >> 4) & 3,
            field_1c,
            field_64,
            field_20,
            vector_28,
            vector_40,
            id_68,
            id_58,
            field_6c,
            field_60,
            vector_70,
            route_90,
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(value)
    }
}

fn vector(input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<[f64; 3]> {
    let mut values = [0.; 3];
    for value in &mut values {
        *value = f64::from(match encoding {
            FloatEncoding::DoublePrecision => input.read_f64()? as f32,
            FloatEncoding::SinglePrecision | FloatEncoding::TypeWidths => input.read_f32()?,
        });
    }
    Ok(values)
}

#[cfg(test)]
mod tests;
