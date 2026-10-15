//! Ordered native Unit suffix, before Creature/Sentient/Person reception.
//! Evidence and unresolved constructor semantics: docs/native-unit.md.
use super::native_condition::NativeConditionList;
use crate::{atom_position::FloatEncoding, protocol::QtReader};
use std::io;

/// Unobserved condition storage stays None until its receive flag occurs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeUnitState {
    pub fields: NativeUnitFields,
    pub conditions: Option<NativeConditionList>,
}

impl NativeUnitState {
    /// Established receive-visible DaUnit constructor subset, not base/owner
    /// allocation. Vector330 belongs to the inherited base and remains unknown
    /// here until Unit reception replaces it. Default stays observation-only.
    pub fn from_default_constructor_subset() -> Self {
        Self {
            fields: NativeUnitFields {
                field_3b0: Some(None),
                field_3a8: Some(0),
                field_39c: Some(0),
                vector_330: None,
            },
            conditions: Some(NativeConditionList::default()),
        }
    }

    pub fn receive(
        &mut self,
        input: &mut QtReader<'_>,
        encoding: FloatEncoding,
    ) -> io::Result<NativeUnitReceive> {
        self.fields.receive(input, encoding, |input| {
            self.conditions
                .get_or_insert_with(Default::default)
                .receive(input, encoding)
        })
    }
}

/// None means not yet observed, not a reconstructed constructor default.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeUnitFields {
    pub field_3b0: Option<Option<String>>,
    pub field_3a8: Option<i32>,
    pub field_39c: Option<u8>,
    pub vector_330: Option<[f64; 3]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeUnitReceive {
    pub flags: u8,
    pub consumed_bytes: usize,
}

impl NativeUnitFields {
    /// Enclosing dispatcher already received the base. The required condition
    /// callback must perform destructive list replacement and consume its real
    /// nested bodies, or return an explicit error. No opaque skipping is valid.
    /// Earlier scalar assignments survive later failures. Vector assignment
    /// waits for all three reads. Rust errors do not emulate Qt stream status.
    pub fn receive(
        &mut self,
        input: &mut QtReader<'_>,
        encoding: FloatEncoding,
        mut conditions: impl FnMut(&mut QtReader<'_>) -> io::Result<()>,
    ) -> io::Result<NativeUnitReceive> {
        let length = input.remaining();
        let flags = input.read_u8()?;
        if flags & 1 != 0 {
            self.field_3b0 = Some(input.read_string()?);
            self.field_3a8 = Some(input.read_i32()?);
            self.field_39c = Some(input.read_u8()?);
        }
        if flags & 2 != 0 {
            conditions(input)?;
        }
        let vector = if flags & 4 == 0 {
            [0.; 3]
        } else {
            [
                float(input, encoding)?,
                float(input, encoding)?,
                float(input, encoding)?,
            ]
        };
        self.vector_330 = Some(vector);
        Ok(NativeUnitReceive {
            flags,
            consumed_bytes: length - input.remaining(),
        })
    }
}

fn float(input: &mut QtReader<'_>, encoding: FloatEncoding) -> io::Result<f64> {
    // The native destination is float even with an eight-byte wire overload.
    match encoding {
        FloatEncoding::DoublePrecision => input.read_f64().map(|value| f64::from(value as f32)),
        FloatEncoding::TypeWidths | FloatEncoding::SinglePrecision => {
            input.read_f32().map(f64::from)
        }
    }
}

#[cfg(test)]
mod tests;
