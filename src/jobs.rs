//! Source-established story job list; execution and other job types are opaque.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct StoryJob {
    pub julian_day_raw: u32,
    pub milliseconds_raw: u32,
    pub time_spec_raw: u8,
    pub author: Option<String>,
    pub brief: Option<String>,
    pub tag_raw: u8,
    pub instance_id: i32,
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JobList {
    pub declared_count: u16,
    pub stories: Vec<StoryJob>,
    pub stop: Option<String>,
    pub unparsed_suffix: Vec<u8>,
}

impl JobList {
    pub(crate) fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        if input.remaining() > 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "job list exceeds byte budget",
            ));
        }
        let count = input.read_u16()? as i16;
        if !(0..=256).contains(&count) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "job list exceeds count budget",
            ));
        }
        let mut list = Self {
            declared_count: count as u16,
            stories: Vec::new(),
            stop: None,
            unparsed_suffix: Vec::new(),
        };
        for _ in 0..count {
            let start = input.remaining_bytes();
            let type_id = input.read_u8()?;
            if type_id != 0x1f {
                list.stop = Some(format!("unknown job type {type_id:#04x}"));
                list.unparsed_suffix = start.to_vec();
                return Ok(list);
            }
            let julian_day_raw = input.read_u32()?;
            let milliseconds_raw = input.read_u32()?;
            let time_spec_raw = input.read_u8()?;
            if time_spec_raw != 1 {
                list.stop = Some("unsupported job datetime profile".into());
                list.unparsed_suffix = start.to_vec();
                return Ok(list);
            }
            let author = input.read_string()?;
            let brief = input.read_string()?;
            let tag_raw = input.read_u8()?;
            if tag_raw != 0 {
                list.stop = Some("unsupported story job tag".into());
                list.unparsed_suffix = start.to_vec();
                return Ok(list);
            }
            let instance_id = input.read_u32()? as i32;
            let title = input.read_string()?;
            list.stories.push(StoryJob {
                julian_day_raw,
                milliseconds_raw,
                time_spec_raw,
                author,
                brief,
                tag_raw,
                instance_id,
                title,
            });
        }
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::QtWriter;

    fn fixture() -> Vec<u8> {
        let mut out = vec![0, 1, 0x1f];
        out.extend(2460000u32.to_be_bytes());
        out.extend(1234567u32.to_be_bytes());
        out.push(1);
        let mut strings = QtWriter::default();
        strings.write_string(Some("Author 🚀")).unwrap();
        strings.write_string(None).unwrap();
        out.extend(strings.into_bytes());
        out.push(0);
        out.extend((-1i32).to_be_bytes());
        let mut title = QtWriter::default();
        title.write_string(Some("Story")).unwrap();
        out.extend(title.into_bytes());
        out
    }

    #[test]
    fn story_fields_and_truncation_have_exact_boundaries() {
        let bytes = fixture();
        let mut reader = QtReader::new(&bytes);
        let jobs = JobList::read(&mut reader).unwrap();
        assert_eq!(reader.remaining(), 0);
        assert_eq!(jobs.declared_count, 1);
        assert_eq!(jobs.stories[0].julian_day_raw, 2460000);
        assert_eq!(jobs.stories[0].milliseconds_raw, 1234567);
        assert_eq!(jobs.stories[0].author, Some("Author 🚀".into()));
        assert_eq!(jobs.stories[0].brief, None);
        assert_eq!(jobs.stories[0].instance_id, -1);
        assert_eq!(jobs.stories[0].title, Some("Story".into()));
        for length in 0..bytes.len() {
            assert!(JobList::read(&mut QtReader::new(&bytes[..length])).is_err());
        }
        let mut extended = bytes.clone();
        extended.extend([9, 8]);
        let mut reader = QtReader::new(&extended);
        JobList::read(&mut reader).unwrap();
        assert_eq!(reader.remaining_bytes(), [9, 8]);
    }

    #[test]
    fn unknown_types_datetime_and_count_limits_stop_safely() {
        let bytes = [0, 1, 0xfe, 9, 8];
        let jobs = JobList::read(&mut QtReader::new(&bytes)).unwrap();
        assert!(jobs.stories.is_empty());
        assert_eq!(jobs.unparsed_suffix, [0xfe, 9, 8]);
        let mut bytes = fixture();
        bytes[11] = 2;
        let jobs = JobList::read(&mut QtReader::new(&bytes)).unwrap();
        assert!(jobs.stories.is_empty());
        assert_eq!(jobs.unparsed_suffix, bytes[2..]);
        assert!(JobList::read(&mut QtReader::new(&[0xff, 0xff])).is_err());
        assert!(JobList::read(&mut QtReader::new(&[1, 1])).is_err());
        assert!(JobList::read(&mut QtReader::new(&vec![0; 1024 * 1024 + 1])).is_err());
    }
}
