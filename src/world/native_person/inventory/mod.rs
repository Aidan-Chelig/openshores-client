use super::*;
use crate::world::native_item::NativeItemValue;

/// Receive observations grouped by unsigned commodity bits. Duplicate entries
/// are retained in wire order within a group, not native hash iteration order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeItemHash(pub BTreeMap<u16, Vec<NativeItemValue>>);

#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "inspect skipped null factories and obsolete item diagnostics"]
pub struct NativeItemHashReceive {
    /// Signed32 overload; native decrement-to-zero uses every raw count bit.
    pub declared_count_bits: u32,
    pub null_factories: Vec<(u32, u8)>,
    pub obsolete_items: Vec<(u32, u16)>,
    pub consumed_bytes: usize,
}

/// DbCommodity::IsObsolete for a received16-bit commodity, zero-extended as at
/// the inventory call site. No guessed catalog bound or signed-short rejection.
pub fn native_commodity_is_obsolete(commodity: u16) -> bool {
    matches!(
        commodity,
        0x36 | 0x46 | 0x47 | 0x67 | 0xed | 0x118 | 0x119 | 0x27b..=0x27e
    )
}

impl NativeItemHash {
    /// Native clears before count. Completed earlier inserts survive later
    /// errors. Known null factories continue, not stop. Supported item bodies
    /// retain their atomic Rust read policy, unlike Qt failed-stream storage.
    pub fn receive_qt7(&mut self, input: &mut QtReader<'_>) -> io::Result<NativeItemHashReceive> {
        let length = input.remaining();
        self.0.clear();
        let declared_count_bits = input.read_u32()?;
        let mut null_factories = vec![];
        let mut obsolete_items = vec![];
        for index in 0..declared_count_bits {
            let mut probe = QtReader::new(input.remaining_bytes());
            let kind = probe.read_u8()?;
            if NativeItemValue::factory_is_null(kind) {
                input.skip(1)?;
                null_factories.push((index, kind));
                continue;
            }
            let item = NativeItemValue::read(input)?.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("native inventory item factory/body {kind} is not implemented"),
                )
            })?;
            let commodity = item.common.commodity as u16;
            if native_commodity_is_obsolete(commodity) {
                obsolete_items.push((index, commodity));
                continue;
            }
            if commodity == 2 {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native inventory commodity2 replacement requires commodity0xa1 construction",
                ));
            }
            self.0.entry(commodity).or_default().push(item);
        }
        Ok(NativeItemHashReceive {
            declared_count_bits,
            null_factories,
            obsolete_items,
            consumed_bytes: length - input.remaining(),
        })
    }
}

#[cfg(test)]
mod tests;
