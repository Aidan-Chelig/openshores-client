//! Read-only layout derived from the supplied sfx.har: index offset followed
//! by QByteArray/qCompress blocks and a compressed QString-indexed directory.
use crate::protocol::QtReader;
use flate2::{Decompress, FlushDecompress, Status};
use std::{collections::BTreeSet, io};

const MAX_ARCHIVE: usize = 64 * 1024 * 1024;
const MAX_ENTRY: usize = 64 * 1024 * 1024;
const MAX_TOTAL_RAW: u64 = 512 * 1024 * 1024;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn word(data: &[u8], offset: usize) -> io::Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| invalid("HAR offset overflow"))?;
    Ok(u32::from_be_bytes(
        data.get(offset..end)
            .ok_or_else(|| invalid("truncated HAR word"))?
            .try_into()
            .unwrap(),
    ))
}

fn block(data: &[u8], offset: usize) -> io::Result<&[u8]> {
    let size = word(data, offset)? as usize;
    let start = offset
        .checked_add(4)
        .ok_or_else(|| invalid("HAR offset overflow"))?;
    let end = start
        .checked_add(size)
        .ok_or_else(|| invalid("HAR block size overflow"))?;
    data.get(start..end)
        .ok_or_else(|| invalid("truncated HAR block"))
}

pub(crate) fn qdecompress(data: &[u8], limit: usize) -> io::Result<Vec<u8>> {
    let expected = word(data, 0)? as usize;
    if expected > limit {
        return Err(invalid("qCompress block exceeds output limit"));
    }
    if data == [0, 0, 0, 0] {
        return Ok(Vec::new());
    }
    let mut decoder = Decompress::new(true);
    let mut output = Vec::with_capacity(expected + 1);
    let status = decoder
        .decompress_vec(&data[4..], &mut output, FlushDecompress::Finish)
        .map_err(|_| invalid("invalid zlib stream"))?;
    if status != Status::StreamEnd
        || output.len() != expected
        || decoder.total_in() != (data.len() - 4) as u64
    {
        return Err(invalid(
            "qCompress size mismatch or trailing compressed bytes",
        ));
    }
    Ok(output)
}

#[derive(Debug)]
pub struct HarEntry {
    pub name: String,
    pub offset: u32,
    pub compressed_bytes: u32,
    pub original_bytes: u32,
}

pub struct HarArchive<'a> {
    data: &'a [u8],
    entries: Vec<HarEntry>,
    pub index_offset: u32,
}

impl<'a> HarArchive<'a> {
    pub fn open(data: &'a [u8]) -> io::Result<Self> {
        if data.len() > MAX_ARCHIVE {
            return Err(invalid("HAR archive exceeds input limit"));
        }
        let index_offset = word(data, 0)?;
        if index_offset < 4 {
            return Err(invalid("HAR index overlaps header"));
        }
        let index = block(data, index_offset as usize)?;
        if index_offset as usize + 4 + index.len() != data.len() {
            return Err(invalid("bytes follow HAR index"));
        }
        let directory = qdecompress(index, 8 * 1024 * 1024)?;
        let mut input = QtReader::new(&directory);
        let count = input.read_u32()?;
        if count > 10_000 {
            return Err(invalid("HAR directory exceeds entry limit"));
        }
        let mut entries = Vec::new();
        let mut names = BTreeSet::new();
        let mut total = 0u64;
        for _ in 0..count {
            let name = input
                .read_string()?
                .filter(|s| !s.is_empty() && !s.contains('\0'))
                .ok_or_else(|| invalid("invalid HAR entry name"))?;
            if !names.insert(name.clone()) {
                return Err(invalid("duplicate HAR entry name"));
            }
            let compressed_bytes = input.read_u32()?;
            let offset = input.read_u32()?;
            let original_bytes = input.read_u32()?;
            if original_bytes as usize > MAX_ENTRY {
                return Err(invalid("HAR entry exceeds output limit"));
            }
            total += u64::from(original_bytes);
            if total > MAX_TOTAL_RAW {
                return Err(invalid("HAR archive exceeds total output budget"));
            }
            let payload = block(data, offset as usize)?;
            if payload.len() != compressed_bytes as usize || word(payload, 0)? != original_bytes {
                return Err(invalid("HAR directory disagrees with entry header"));
            }
            entries.push(HarEntry {
                name,
                offset,
                compressed_bytes,
                original_bytes,
            });
        }
        if input.remaining() != 0 {
            return Err(invalid("trailing HAR directory bytes"));
        }
        let mut spans: Vec<_> = entries.iter().collect();
        spans.sort_by_key(|entry| entry.offset);
        let mut end = 4u64;
        for entry in spans {
            if u64::from(entry.offset) != end {
                return Err(invalid("HAR entry ranges overlap or leave gaps"));
            }
            end += 4 + u64::from(entry.compressed_bytes);
            if end > u64::from(index_offset) {
                return Err(invalid("HAR entry overlaps index"));
            }
        }
        if end != u64::from(index_offset) {
            return Err(invalid("HAR entries do not cover the data section"));
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self {
            data,
            entries,
            index_offset,
        })
    }

    pub fn entries(&self) -> &[HarEntry] {
        &self.entries
    }

    pub fn read_entry(&self, name: &str) -> io::Result<Vec<u8>> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HAR entry not found"))?;
        qdecompress(block(self.data, entry.offset as usize)?, MAX_ENTRY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::QtWriter;
    use flate2::{Compression, write::ZlibEncoder};
    use std::io::Write;

    fn compressed(raw: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(raw).unwrap();
        let mut data = (raw.len() as u32).to_be_bytes().to_vec();
        data.extend(encoder.finish().unwrap());
        data
    }

    fn archive() -> Vec<u8> {
        let raw = b"synthetic audio bytes";
        let payload = compressed(raw);
        let mut data = vec![0; 4];
        data.extend((payload.len() as u32).to_be_bytes());
        data.extend(&payload);
        let index_offset = data.len() as u32;
        data[..4].copy_from_slice(&index_offset.to_be_bytes());
        let mut index = QtWriter::default();
        index.write_u32(1);
        index.write_string(Some("sound.wav")).unwrap();
        index.write_u32(payload.len() as u32);
        index.write_u32(4);
        index.write_u32(raw.len() as u32);
        let index = compressed(&index.into_bytes());
        data.extend((index.len() as u32).to_be_bytes());
        data.extend(index);
        data
    }

    #[test]
    fn bounds_and_compression_errors() {
        let data = archive();
        assert_eq!(
            HarArchive::open(&data)
                .unwrap()
                .read_entry("sound.wav")
                .unwrap(),
            b"synthetic audio bytes"
        );
        assert!(
            HarArchive::open(&data)
                .unwrap()
                .read_entry("absent")
                .is_err()
        );
        for end in 0..data.len() {
            assert!(HarArchive::open(&data[..end]).is_err());
        }
        let mut extra = data.clone();
        extra.push(0);
        assert!(HarArchive::open(&extra).is_err());
        let mut broken = data;
        broken[0..4].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(HarArchive::open(&broken).is_err());
        let mut wrong = compressed(b"abc");
        wrong[..4].copy_from_slice(&2u32.to_be_bytes());
        assert!(qdecompress(&wrong, 1024).is_err());
        let mut trailing = compressed(b"abc");
        trailing.push(0);
        assert!(qdecompress(&trailing, 1024).is_err());
        assert!(qdecompress(&compressed(b"abc"), 2).is_err());
        let compressed = compressed(b"abc");
        for end in 0..compressed.len() {
            assert!(qdecompress(&compressed[..end], 1024).is_err());
        }
    }
}
