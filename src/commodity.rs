//! Server-provided DbCommodity labels/resources; numeric gameplay fields stay raw.
use crate::protocol::QtReader;
use std::{collections::BTreeMap, io};

const MAX_ROW_BYTES: usize = 1024 * 1024;
const MAX_ROWS: usize = 4096;
const MAX_CATALOG_BYTES: usize = 16 * 1024 * 1024;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commodity {
    pub id: i16,
    /// Raw source commodity flags, not item-instance flags.
    pub flags: u32,
    pub description: Option<String>,
    pub bitmap: Option<String>,
    pub model: Option<String>,
    pub name: Option<String>,
    /// Complete row excluding opcode, including uninterpreted numeric fields.
    pub raw: Vec<u8>,
    /// Extra bytes beyond the source-established row layout are preserved.
    pub unparsed_suffix: Vec<u8>,
}

impl Commodity {
    pub fn decode(packet: &[u8]) -> io::Result<Self> {
        if packet.len() > MAX_ROW_BYTES {
            return Err(invalid("commodity row exceeds byte budget"));
        }
        let mut input = QtReader::new(packet);
        if input.read_u8()? != 0x2b {
            return Err(invalid("expected DbCommodity opcode"));
        }
        let id = input.read_u16()? as i16;
        // gd_tables.py::_COMMODITY_LAYOUT and db_static_tables.py's writer
        // establish these fixed-width blocks. Do not infer their numeric meaning.
        input.skip(53)?;
        let flags = input.read_u32()?;
        let description = input.read_string()?;
        input.skip(2)?;
        let bitmap = input.read_string()?;
        let model = input.read_string()?;
        input.skip(29)?;
        let name = input.read_string()?;
        input.skip(50)?;
        Ok(Self {
            id,
            flags,
            description,
            bitmap,
            model,
            name,
            raw: packet[1..].to_vec(),
            unparsed_suffix: input.remaining_bytes().to_vec(),
        })
    }
}

#[derive(Default)]
pub struct CommodityCatalog {
    rows: BTreeMap<i16, Commodity>,
    bytes: usize,
}

impl CommodityCatalog {
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub fn get(&self, id: i32) -> Option<&Commodity> {
        self.rows.get(&i16::try_from(id).ok()?)
    }
    pub fn rows(&self) -> impl Iterator<Item = &Commodity> {
        self.rows.values()
    }
    /// Whole-row replacement in receive order; invalid input leaves state intact.
    pub fn ingest(&mut self, packet: &[u8]) -> io::Result<()> {
        let row = Commodity::decode(packet)?;
        let previous = self.rows.get(&row.id);
        if previous.is_none() && self.rows.len() >= MAX_ROWS {
            return Err(invalid("commodity catalog row budget exceeded"));
        }
        let bytes = self.bytes - previous.map_or(0, |row| row.raw.len()) + row.raw.len();
        if bytes > MAX_CATALOG_BYTES {
            return Err(invalid("commodity catalog byte budget exceeded"));
        }
        self.rows.insert(row.id, row);
        self.bytes = bytes;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::QtWriter;
    fn packet(id: i16, name: Option<&str>) -> Vec<u8> {
        let mut out = QtWriter::default();
        out.write_u8(0x2b);
        out.write_u16(id as u16);
        for _ in 0..57 {
            out.write_u8(0);
        }
        out.write_string(Some("Description 🌊")).unwrap();
        out.write_u16(0);
        out.write_string(Some(":data/icon.png")).unwrap();
        out.write_string(None).unwrap();
        for _ in 0..29 {
            out.write_u8(0);
        }
        out.write_string(name).unwrap();
        for _ in 0..50 {
            out.write_u8(0);
        }
        out.into_bytes()
    }
    #[test]
    fn labels_preserve_unicode_null_empty_signed_ids_and_raw_bytes() {
        for name in [None, Some(""), Some("Калашников 🌊")] {
            let mut bytes = packet(-2, name);
            bytes.extend([7, 8]);
            let row = Commodity::decode(&bytes).unwrap();
            assert_eq!(row.id, -2);
            assert_eq!(row.name.as_deref(), name);
            assert_eq!(row.description.as_deref(), Some("Description 🌊"));
            assert_eq!(row.bitmap.as_deref(), Some(":data/icon.png"));
            assert_eq!(row.model, None);
            assert_eq!(row.raw, bytes[1..]);
            assert_eq!(row.unparsed_suffix, [7, 8]);
        }
    }
    #[test]
    fn truncation_and_invalid_replacement_leave_catalog_unchanged() {
        let bytes = packet(31, Some("Pistol"));
        let mut catalog = CommodityCatalog::default();
        catalog.ingest(&bytes).unwrap();
        for end in 0..bytes.len() {
            assert!(catalog.ingest(&bytes[..end]).is_err());
        }
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog.get(31).unwrap().name.as_deref(), Some("Pistol"));
        assert!(catalog.get(65567).is_none());
        catalog.ingest(&packet(31, None)).unwrap();
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog.get(31).unwrap().name, None);
        assert_eq!(catalog.bytes, catalog.get(31).unwrap().raw.len());
    }
    #[test]
    fn budgets_and_wrong_opcode_are_rejected_atomically() {
        let mut catalog = CommodityCatalog::default();
        let mut wrong = packet(1, Some("one"));
        wrong[0] = 0x2c;
        assert!(catalog.ingest(&wrong).is_err());
        assert!(catalog.ingest(&vec![0; MAX_ROW_BYTES + 1]).is_err());
        for id in 0..MAX_ROWS {
            catalog.ingest(&packet(id as i16, None)).unwrap();
        }
        assert!(catalog.ingest(&packet(MAX_ROWS as i16, None)).is_err());
        catalog.ingest(&packet(0, Some("replacement"))).unwrap();
        let before = catalog.bytes;
        catalog.bytes = MAX_CATALOG_BYTES;
        assert!(
            catalog
                .ingest(&packet(0, Some("larger replacement")))
                .is_err()
        );
        assert_eq!(catalog.get(0).unwrap().name.as_deref(), Some("replacement"));
        catalog.bytes = before;
    }
}
