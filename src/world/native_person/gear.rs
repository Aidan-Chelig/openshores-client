use super::*;
use crate::world::native_item::NativeItemValue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeGearSlot {
    pub field_08: u8,
    pub field_09: u8,
    pub item: NativeItemValue,
}

impl NativeGearSlot {
    /// Fresh-value atomicity on Rust failure, not native partial-write parity.
    /// None means a verified null factory. Other unimplemented factory types
    /// and commodity2 replacement fail explicitly, never guess/skip a body.
    pub fn read(input: &mut QtReader<'_>) -> io::Result<Option<Self>> {
        Self::read_with_profile(input, FloatEncoding::TypeWidths)
    }

    pub fn read_with_profile(
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<Option<Self>> {
        let mut record = QtReader::new(input.remaining_bytes());
        let packed = record.read_u8()?;
        let mut probe = QtReader::new(record.remaining_bytes());
        let kind = probe.read_u8()?;
        let result = if NativeItemValue::factory_is_null(kind) {
            record.skip(1)?;
            None
        } else {
            let item =
                NativeItemValue::read_with_profile(&mut record, profile)?.ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::Unsupported,
                        format!("native gear item factory type {kind}"),
                    )
                })?;
            if item.common.commodity == 2 {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native gear commodity2 FixItem requires commodity0xa1 construction",
                ));
            }
            Some(Self {
                field_08: packed & 0x0f,
                field_09: packed >> 4,
                item,
            })
        };
        input.skip(input.remaining() - record.remaining())?;
        Ok(result)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeGear(pub Vec<NativeGearSlot>);

#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "inspect null-slot diagnostics; gear reception does not recompute Person weight"]
pub struct NativeGearReceive {
    pub wire_count: u8,
    /// Zero-based wire slots whose null item was warned about and not appended.
    pub null_slots: Vec<u8>,
    pub consumed_bytes: usize,
}

impl NativeGear {
    /// Clear before count, append completed non-null slots in order (including
    /// duplicates). No count-based allocation. Earlier appends survive later
    /// errors. Person must separately recompute actual native Gear::Weight.
    pub fn receive(&mut self, input: &mut QtReader<'_>) -> io::Result<NativeGearReceive> {
        self.receive_with_profile(input, FloatEncoding::TypeWidths)
    }

    pub fn receive_with_profile(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<NativeGearReceive> {
        let length = input.remaining();
        self.0.clear();
        let wire_count = input.read_u8()?;
        let mut null_slots = vec![];
        for index in 0..wire_count {
            if let Some(slot) = NativeGearSlot::read_with_profile(input, profile)? {
                self.0.push(slot);
            } else {
                null_slots.push(index);
            }
        }
        Ok(NativeGearReceive {
            wire_count,
            null_slots,
            consumed_bytes: length - input.remaining(),
        })
    }
}

#[cfg(test)]
mod tests;
