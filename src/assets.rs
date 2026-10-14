//! GD construction table from reference server gameplay/gd_tables.py.
use crate::protocol::QtReader;
use std::io;

#[derive(Debug)]
pub struct ConstructionProcess {
    pub id: u8,
    pub name: String,
    pub industry: u8,
    pub unknown: [u8; 2],
    pub terrain: u8,
    pub radius: u8,
}

/// Validates the supplied server GD layout before returning rows.
pub fn construction_processes(bytes: &[u8]) -> io::Result<Vec<ConstructionProcess>> {
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "GD does not match the reference server construction-table layout",
        )
    };
    let mut input = QtReader::new(bytes.get(397_280..).ok_or_else(invalid)?);
    let mut rows = Vec::new();
    for id in 1..=u8::MAX {
        if input.remaining() == 0 || input.read_u8()? != id {
            break;
        }
        let name = input
            .read_string()?
            .filter(|name| {
                !name.is_empty() && name.len() <= 400 && name.chars().all(|c| !c.is_control())
            })
            .ok_or_else(invalid)?;
        rows.push(ConstructionProcess {
            id,
            name,
            industry: input.read_u8()?,
            unknown: [input.read_u8()?, input.read_u8()?],
            terrain: input.read_u8()?,
            radius: input.read_u8()?,
        });
    }
    if rows.len() < 50 {
        return Err(invalid());
    }
    for (index, name, industry) in [
        (0, "Town Square", 1),
        (1, "Dirt", 2),
        (2, "Asphalt", 2),
        (3, "Concrete", 2),
        (4, "Graded Area", 2),
    ] {
        if rows[index].name != name || rows[index].industry != industry {
            return Err(invalid());
        }
    }
    for (index, radius) in [(1, 5), (2, 8), (3, 8), (4, 5)] {
        if rows[index].radius != radius {
            return Err(invalid());
        }
    }
    Ok(rows)
}
