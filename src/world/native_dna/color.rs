//! Raw QColor storage for the verified native scene QDataStream version 7.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeColorQt7 {
    pub spec: i32,
    pub components: [u16; 5],
}

impl NativeColorQt7 {
    /// Version >6 reads a signed byte and five shorts regardless of color
    /// specification. Do not substitute RGBA bytes or infer a component color
    /// space before interpreting spec. Atomic failure is Rust-only safety.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Self> {
        let mut record = QtReader::new(input.remaining_bytes());
        let spec = i32::from(record.read_u8()? as i8);
        let mut components = [0; 5];
        for component in &mut components {
            *component = record.read_u16()?;
        }
        input.skip(input.remaining() - record.remaining())?;
        Ok(Self { spec, components })
    }
}
