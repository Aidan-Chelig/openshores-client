//! Offline Qt resource arrays in the supplied, symbol-bearing PE DLL.
//! Version must be established from registration code, not guessed from bytes.
use std::{collections::BTreeSet, io, ops::Range};
mod registration;

const MAX_FILE: usize = 128 * 1024 * 1024;
const MAX_ENTRY: usize = 64 * 1024 * 1024;
const MAX_NODES: usize = 100_000;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal synthetic PE/COFF image, one root and one file, no executable code.
    fn fixture() -> Vec<u8> {
        let mut bytes = vec![0; 640];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[70..72].copy_from_slice(&1u16.to_le_bytes());
        bytes[76..80].copy_from_slice(&400u32.to_le_bytes());
        bytes[80..84].copy_from_slice(&3u32.to_le_bytes());
        // No optional header; sole section's raw bytes start at 128.
        bytes[104..108].copy_from_slice(&200u32.to_le_bytes());
        bytes[108..112].copy_from_slice(&128u32.to_le_bytes());
        // Root: directory, one child at node 1.
        bytes[132..134].copy_from_slice(&2u16.to_be_bytes());
        bytes[134..138].copy_from_slice(&1u32.to_be_bytes());
        bytes[138..142].copy_from_slice(&1u32.to_be_bytes());
        // File at 150, name at 172, payload at 188.
        bytes[158..160].copy_from_slice(&1u16.to_be_bytes());
        bytes[172..174].copy_from_slice(&4u16.to_be_bytes());
        for (index, unit) in "test".encode_utf16().enumerate() {
            bytes[178 + index * 2..180 + index * 2].copy_from_slice(&unit.to_be_bytes());
        }
        bytes[188..192].copy_from_slice(&3u32.to_be_bytes());
        bytes[192..195].copy_from_slice(b"abc");
        let mut strings = vec![0; 4];
        for (index, (name, relative)) in [
            ("_ZL16qt_resource_data", 60u32),
            ("_ZL16qt_resource_name", 44),
            ("_ZL18qt_resource_struct", 0),
        ]
        .into_iter()
        .enumerate()
        {
            let symbol = 400 + index * 18;
            bytes[symbol + 4..symbol + 8].copy_from_slice(&(strings.len() as u32).to_le_bytes());
            bytes[symbol + 8..symbol + 12].copy_from_slice(&relative.to_le_bytes());
            bytes[symbol + 12..symbol + 14].copy_from_slice(&1u16.to_le_bytes());
            strings.extend(name.as_bytes());
            strings.push(0);
        }
        let length = strings.len() as u32;
        strings[..4].copy_from_slice(&length.to_le_bytes());
        bytes[454..454 + strings.len()].copy_from_slice(&strings);
        bytes.truncate(454 + strings.len());
        bytes
    }

    #[test]
    fn synthetic_inventory_and_truncation() {
        let bytes = fixture();
        let resources = QtResources::open(&bytes, 3).unwrap();
        assert_eq!(resources.pool_count, 1);
        assert_eq!(resources.entries()[0].path, ":/test");
        assert_eq!(resources.read_entry(0).unwrap(), b"abc");
        assert!(resources.read_entry(1).is_err());
        assert!(QtResources::open(&bytes, 0).is_err());
        for end in 0..bytes.len() {
            assert!(
                QtResources::open(&bytes[..end], 3).is_err(),
                "accepted truncation at {end}"
            );
        }
    }

    #[test]
    fn invalid_tree_names_ranges_and_symbols() {
        for (offset, replacement) in [
            (138, 0u32.to_be_bytes().to_vec()), // Root references itself.
            (134, u32::MAX.to_be_bytes().to_vec()),
            (154, 4u16.to_be_bytes().to_vec()), // Unsupported zstd.
            (160, u32::MAX.to_be_bytes().to_vec()), // Payload offset.
            (188, u32::MAX.to_be_bytes().to_vec()), // Payload size.
            (178, 0xd800u16.to_be_bytes().to_vec()), // Unpaired surrogate.
            (178, ('/' as u16).to_be_bytes().to_vec()),
            (412, 2u16.to_le_bytes().to_vec()), // Nonexistent symbol section.
            (417, vec![255]),                   // Oversized auxiliary table.
        ] {
            let mut bytes = fixture();
            bytes[offset..offset + replacement.len()].copy_from_slice(&replacement);
            assert!(
                QtResources::open(&bytes, 3).is_err(),
                "accepted mutation at {offset}"
            );
        }
    }
}
fn slice(bytes: &[u8], offset: usize, size: usize) -> io::Result<&[u8]> {
    bytes
        .get(
            offset
                ..offset
                    .checked_add(size)
                    .ok_or_else(|| invalid("resource offset overflow"))?,
        )
        .ok_or_else(|| invalid("resource range outside backing bytes"))
}
fn le16(bytes: &[u8], offset: usize) -> io::Result<u16> {
    Ok(u16::from_le_bytes(
        slice(bytes, offset, 2)?.try_into().unwrap(),
    ))
}
fn le32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        slice(bytes, offset, 4)?.try_into().unwrap(),
    ))
}
fn be16(bytes: &[u8], offset: usize) -> io::Result<u16> {
    Ok(u16::from_be_bytes(
        slice(bytes, offset, 2)?.try_into().unwrap(),
    ))
}
fn be32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    Ok(u32::from_be_bytes(
        slice(bytes, offset, 4)?.try_into().unwrap(),
    ))
}

#[derive(Debug)]
pub struct ResourceEntry {
    pub pool: usize,
    pub path: String,
    pub country: u16,
    pub language: u16,
    pub flags: u16,
    pub stored_bytes: usize,
    pub original_bytes: usize,
    pub payload_range: Range<usize>,
}

pub struct QtResources<'a> {
    bytes: &'a [u8],
    entries: Vec<ResourceEntry>,
    pub pool_count: usize,
}

/// Discover repeated data/name/tree symbol triplets emitted by this MinGW build.
/// This is deliberately not a general PE loader or stripped-binary scanner.
fn pools(bytes: &[u8]) -> io::Result<Vec<[Range<usize>; 3]>> {
    if bytes.len() > MAX_FILE || slice(bytes, 0, 2)? != b"MZ" {
        return Err(invalid("expected bounded PE image"));
    }
    let pe = le32(bytes, 0x3c)? as usize;
    if slice(bytes, pe, 4)? != b"PE\0\0" {
        return Err(invalid("invalid PE signature"));
    }
    let header = pe + 4;
    let section_count = le16(bytes, header + 2)? as usize;
    if section_count == 0 || section_count > 96 {
        return Err(invalid("unsupported PE section count"));
    }
    let sections = header + 20 + le16(bytes, header + 16)? as usize;
    slice(bytes, sections, section_count * 40)?;
    let symbol_offset = le32(bytes, header + 8)? as usize;
    let symbol_count = le32(bytes, header + 12)? as usize;
    if symbol_offset == 0 || symbol_count == 0 || symbol_count > 1_000_000 {
        return Err(invalid("missing or oversized COFF symbol table"));
    }
    slice(bytes, symbol_offset, symbol_count * 18)?;
    let strings_offset = symbol_offset + symbol_count * 18;
    let strings_length = le32(bytes, strings_offset)? as usize;
    if strings_length < 4 {
        return Err(invalid("invalid COFF string table"));
    }
    let strings = slice(bytes, strings_offset, strings_length)?;
    let mut arrays = Vec::new();
    let mut index = 0;
    while index < symbol_count {
        let symbol = slice(bytes, symbol_offset + index * 18, 18)?;
        let name_bytes = if le32(symbol, 0)? == 0 {
            let offset = le32(symbol, 4)? as usize;
            if offset < 4 {
                return Err(invalid("invalid COFF symbol name offset"));
            }
            strings
                .get(offset..)
                .ok_or_else(|| invalid("COFF name outside string table"))?
        } else {
            &symbol[..8]
        };
        let name = &name_bytes[..name_bytes
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(name_bytes.len())];
        let kind = match name {
            b"_ZL16qt_resource_data" => Some(2),
            b"_ZL16qt_resource_name" => Some(1),
            b"_ZL18qt_resource_struct" => Some(0),
            _ => None,
        };
        if let Some(kind) = kind {
            let section = le16(symbol, 12)? as usize;
            if section == 0 || section > section_count {
                return Err(invalid("resource symbol has invalid section"));
            }
            let section_header = sections + (section - 1) * 40;
            let raw_size = le32(bytes, section_header + 16)? as usize;
            let raw_offset = le32(bytes, section_header + 20)? as usize;
            slice(bytes, raw_offset, raw_size)?;
            let relative = le32(symbol, 8)? as usize;
            if relative >= raw_size {
                return Err(invalid("resource symbol outside section"));
            }
            arrays.push((kind, raw_offset + relative, raw_offset + raw_size));
        }
        index += 1 + symbol[17] as usize;
        if index > symbol_count {
            return Err(invalid("COFF auxiliary symbols exceed table"));
        }
    }
    if arrays.is_empty() || arrays.len() % 3 != 0 || arrays.len() > 300 {
        return Err(invalid("missing or incomplete Qt resource symbol triplets"));
    }
    let mut result = Vec::new();
    for triple in arrays.chunks_exact(3) {
        if triple.iter().map(|v| v.0).collect::<Vec<_>>() != [2, 1, 0] {
            return Err(invalid("unsupported Qt symbol ordering"));
        }
        let (data, names, tree) = (triple[0], triple[1], triple[2]);
        if tree.1 >= names.1 || names.1 >= data.1 || tree.2 != data.2 || names.2 != data.2 {
            return Err(invalid("invalid Qt array ordering or section"));
        }
        let end = arrays
            .iter()
            .filter(|v| v.1 > data.1 && v.2 == data.2)
            .map(|v| v.1)
            .min()
            .unwrap_or(data.2);
        result.push([tree.1..names.1, names.1..data.1, data.1..end]);
    }
    Ok(result)
}

impl<'a> QtResources<'a> {
    pub fn open(bytes: &'a [u8], version: u8) -> io::Result<Self> {
        if !(1..=3).contains(&version) {
            return Err(invalid("unsupported Qt resource version"));
        }
        let pools = pools(bytes)?;
        Self::from_pools(bytes, version, pools)
    }

    /// Read the observed MSVC x64 resource-registration layout without loading code.
    pub fn open_registered(bytes: &'a [u8]) -> io::Result<Self> {
        let (version, pools) = registration::pools(bytes)?;
        Self::from_pools(bytes, version, pools)
    }

    fn from_pools(bytes: &'a [u8], version: u8, pools: Vec<[Range<usize>; 3]>) -> io::Result<Self> {
        let mut result = Self {
            bytes,
            entries: Vec::new(),
            pool_count: pools.len(),
        };
        let stride = if version == 1 { 14 } else { 22 };
        let mut identities = BTreeSet::new();
        let mut visited_total = 0;
        let mut path_bytes = 0usize;
        let mut total_raw = 0u64;
        for (pool, ranges) in pools.iter().enumerate() {
            let tree = &bytes[ranges[0].clone()];
            let names = &bytes[ranges[1].clone()];
            let data = &bytes[ranges[2].clone()];
            let mut pending = vec![(0usize, String::from(":"), 0usize)];
            let mut visited = BTreeSet::new();
            while let Some((index, parent, depth)) = pending.pop() {
                visited_total += 1;
                if visited_total > MAX_NODES || depth > 64 || !visited.insert(index) {
                    return Err(invalid(
                        "Qt resource cycle, shared node, or traversal limit",
                    ));
                }
                let node = slice(
                    tree,
                    index
                        .checked_mul(stride)
                        .ok_or_else(|| invalid("node overflow"))?,
                    stride,
                )?;
                let path = if index == 0 {
                    parent
                } else {
                    let offset = be32(node, 0)? as usize;
                    let length = be16(names, offset)? as usize;
                    let raw = slice(names, offset + 6, length * 2)?;
                    let units: Vec<_> = raw
                        .chunks_exact(2)
                        .map(|p| u16::from_be_bytes(p.try_into().unwrap()))
                        .collect();
                    let name = String::from_utf16(&units)
                        .map_err(|_| invalid("invalid UTF-16 resource name"))?;
                    if name.is_empty()
                        || name == "."
                        || name == ".."
                        || name.contains(['/', '\\', '\0'])
                    {
                        return Err(invalid("invalid resource path component"));
                    }
                    let path = format!("{parent}/{name}");
                    if path.len() > 4096 {
                        return Err(invalid("resource path exceeds limit"));
                    }
                    path
                };
                let flags = be16(node, 4)?;
                if flags & 4 != 0 {
                    return Err(invalid("zstd resource inventory is not implemented"));
                }
                if flags & !7 != 0 || flags & 3 == 3 || flags & 6 == 6 || flags & 5 == 5 {
                    return Err(invalid("invalid Qt resource flags"));
                }
                if index == 0 && flags != 2 {
                    return Err(invalid("Qt root is not a directory"));
                }
                if flags & 2 != 0 {
                    let count = be32(node, 6)? as usize;
                    let first = be32(node, 10)? as usize;
                    if count > MAX_NODES
                        || first
                            .checked_add(count)
                            .is_none_or(|end| end > tree.len() / stride)
                    {
                        return Err(invalid("Qt child range exceeds tree"));
                    }
                    if pending.len() + visited_total + count > MAX_NODES {
                        return Err(invalid("Qt resource traversal exceeds node budget"));
                    }
                    for child in (first..first + count).rev() {
                        path_bytes += path.len();
                        if path_bytes > 16 * 1024 * 1024 {
                            return Err(invalid("Qt resource paths exceed memory budget"));
                        }
                        pending.push((child, path.clone(), depth + 1));
                    }
                } else {
                    let country = be16(node, 6)?;
                    let language = be16(node, 8)?;
                    if !identities.insert((pool, path.clone(), country, language)) {
                        return Err(invalid("duplicate resource identity"));
                    }
                    let offset = be32(node, 10)? as usize;
                    let stored_bytes = be32(data, offset)? as usize;
                    let payload = slice(data, offset + 4, stored_bytes)?;
                    let original_bytes = if flags & 1 != 0 {
                        be32(payload, 0)? as usize
                    } else {
                        stored_bytes
                    };
                    if stored_bytes > MAX_ENTRY || original_bytes > MAX_ENTRY {
                        return Err(invalid("resource exceeds entry limit"));
                    }
                    total_raw += original_bytes as u64;
                    if total_raw > 512 * 1024 * 1024 {
                        return Err(invalid("Qt resources exceed total output budget"));
                    }
                    let start = ranges[2].start + offset + 4;
                    result.entries.push(ResourceEntry {
                        pool,
                        path,
                        country,
                        language,
                        flags,
                        stored_bytes,
                        original_bytes,
                        payload_range: start..start + stored_bytes,
                    });
                }
            }
        }
        Ok(result)
    }
    pub fn entries(&self) -> &[ResourceEntry] {
        &self.entries
    }
    /// Index disambiguates pools and locale variants. No locale selection is inferred.
    pub fn read_entry(&self, index: usize) -> io::Result<Vec<u8>> {
        let entry = self
            .entries
            .get(index)
            .ok_or_else(|| invalid("resource index out of range"))?;
        let payload = &self.bytes[entry.payload_range.clone()];
        match entry.flags {
            0 => Ok(payload.to_vec()),
            1 => crate::har::qdecompress(payload, MAX_ENTRY),
            _ => Err(invalid("zstd resource extraction is not implemented")),
        }
    }
}
