//! Narrow, source-bound MSVC x64 profile: three RIP-relative LEAs, version,
//! direct call to an import thunk for Qt's qRegisterResourceData. Never executes.
use super::{MAX_FILE, invalid, le16, le32, slice};
use std::{collections::BTreeSet, io, ops::Range};
#[cfg(test)]
#[path = "registration_tests.rs"]
mod tests;

struct Section {
    rva: usize,
    raw: Range<usize>,
    executable: bool,
}
struct Pe<'a> {
    bytes: &'a [u8],
    sections: Vec<Section>,
    imports: usize,
}
impl<'a> Pe<'a> {
    fn open(bytes: &'a [u8]) -> io::Result<Self> {
        if bytes.len() > MAX_FILE || slice(bytes, 0, 2)? != b"MZ" {
            return Err(invalid("expected bounded PE image"));
        }
        let pe = le32(bytes, 0x3c)? as usize;
        if slice(bytes, pe, 4)? != b"PE\0\0" || le16(bytes, pe + 4)? != 0x8664 {
            return Err(invalid("resource registration requires x64 PE"));
        }
        let count = le16(bytes, pe + 6)? as usize;
        let optional = pe + 24;
        let size = le16(bytes, pe + 20)? as usize;
        if count == 0 || count > 96 || size < 128 || le16(bytes, optional)? != 0x20b {
            return Err(invalid("invalid PE32+ headers"));
        }
        slice(bytes, optional, size)?;
        if le32(bytes, optional + 108)? < 2 {
            return Err(invalid("PE has no import directory"));
        }
        let imports = le32(bytes, optional + 120)? as usize;
        let mut sections = Vec::new();
        for i in 0..count {
            let header = optional + size + i * 40;
            slice(bytes, header, 40)?;
            let start = le32(bytes, header + 20)? as usize;
            let length = le32(bytes, header + 16)? as usize;
            slice(bytes, start, length)?;
            sections.push(Section {
                rva: le32(bytes, header + 12)? as usize,
                raw: start..start + length,
                executable: le32(bytes, header + 36)? & 0x20000000 != 0,
            });
        }
        Ok(Self {
            bytes,
            sections,
            imports,
        })
    }
    fn map(&self, rva: usize) -> io::Result<usize> {
        self.sections
            .iter()
            .find_map(|s| {
                rva.checked_sub(s.rva)
                    .filter(|d| *d < s.raw.len())
                    .map(|d| s.raw.start + d)
            })
            .ok_or_else(|| invalid("resource RVA outside file-backed sections"))
    }
    fn relative(&self, offset: usize, displacement: usize, length: usize) -> io::Result<usize> {
        let s = self
            .sections
            .iter()
            .find(|s| s.raw.contains(&offset))
            .ok_or_else(|| invalid("instruction outside sections"))?;
        let delta = i32::from_le_bytes(
            slice(self.bytes, offset + displacement, 4)?
                .try_into()
                .unwrap(),
        );
        let target = (s.rva + offset - s.raw.start + length) as i64 + i64::from(delta);
        usize::try_from(target).map_err(|_| invalid("negative RIP-relative target"))
    }
    fn registration_imports(&self) -> io::Result<BTreeSet<usize>> {
        let mut found = BTreeSet::new();
        if self.imports == 0 {
            return Ok(found);
        }
        for descriptor in 0..1024 {
            let pos = self.map(self.imports + descriptor * 20)?;
            let names = le32(self.bytes, pos)? as usize;
            let library = le32(self.bytes, pos + 12)?;
            let iat = le32(self.bytes, pos + 16)? as usize;
            if names == 0 && library == 0 && iat == 0 {
                return Ok(found);
            }
            if names == 0 {
                return Err(invalid("unsupported bound import table"));
            }
            for index in 0..100_000 {
                let thunk = self.map(names + index * 8)?;
                let value = u64::from_le_bytes(slice(self.bytes, thunk, 8)?.try_into().unwrap());
                if value == 0 {
                    break;
                }
                if value >> 63 != 0 {
                    continue;
                }
                let name = self
                    .map(usize::try_from(value).map_err(|_| invalid("import RVA overflow"))?)?
                    + 2;
                let raw = slice(
                    self.bytes,
                    name,
                    64.min(self.bytes.len().saturating_sub(name)),
                )?;
                if raw.starts_with(b"?qRegisterResourceData@@YA_NHPEBE00@Z\0") {
                    found.insert(iat + index * 8);
                }
                if index == 99_999 {
                    return Err(invalid("import thunk budget exceeded"));
                }
            }
        }
        Err(invalid("import descriptor budget exceeded"))
    }
}

pub(super) fn pools(bytes: &[u8]) -> io::Result<(u8, Vec<[Range<usize>; 3]>)> {
    let pe = Pe::open(bytes)?;
    let imports = pe.registration_imports()?;
    if imports.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no Qt registration import",
        ));
    }
    let mut registrations = BTreeSet::new();
    for section in pe.sections.iter().filter(|s| s.executable) {
        for offset in section.raw.start..section.raw.end.saturating_sub(31) {
            let code = &bytes[offset..offset + 31];
            if code[..3] != [0x4c, 0x8d, 0x0d]
                || code[7] != 0xb9
                || code[12..15] != [0x4c, 0x8d, 0x05]
                || code[19..22] != [0x48, 0x8d, 0x15]
                || code[26] != 0xe8
            {
                continue;
            }
            let target = pe.map(pe.relative(offset + 26, 1, 5)?)?;
            if slice(bytes, target, 2)? != [0xff, 0x25] {
                continue;
            }
            if !imports.contains(&pe.relative(target, 2, 6)?) {
                continue;
            }
            let version = le32(bytes, offset + 8)?;
            if !(1..=3).contains(&version) {
                return Err(invalid("unsupported registered Qt resource version"));
            }
            let data = pe.map(pe.relative(offset, 3, 7)?)?;
            let names = pe.map(pe.relative(offset + 12, 3, 7)?)?;
            let tree = pe.map(pe.relative(offset + 19, 3, 7)?)?;
            registrations.insert((version as u8, tree, names, data));
            if registrations.len() > 128 {
                return Err(invalid("resource pool budget exceeded"));
            }
        }
    }
    let version = registrations
        .first()
        .ok_or_else(|| invalid("no supported Qt resource registrations; expected MSVC x64 layout"))?
        .0;
    let mut result = Vec::new();
    for &(v, tree, names, data) in &registrations {
        if v != version {
            return Err(invalid("mixed registered Qt resource versions"));
        }
        let mut ranges = Vec::new();
        for start in [tree, names, data] {
            let section = pe
                .sections
                .iter()
                .find(|s| s.raw.contains(&start) && !s.executable)
                .ok_or_else(|| invalid("resource array not in a data section"))?;
            let end = registrations
                .iter()
                .flat_map(|r| [r.1, r.2, r.3])
                .filter(|p| *p > start && *p < section.raw.end)
                .min()
                .unwrap_or(section.raw.end);
            ranges.push(start..end);
        }
        result.push(ranges.try_into().unwrap());
    }
    Ok((version, result))
}
