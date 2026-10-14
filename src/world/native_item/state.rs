use super::{NativeItemBody, NativeItemValue, OwnershipUpdate};

/// Established item-field observations, not guessed native constructor defaults.
/// None means unobserved/unknown; Qt-null strings are Some(None).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeItemState {
    pub item: Option<NativeItemValue>,
    /// None is unobserved; false is an established null native item pointer.
    /// Some(true) accompanies a decoded received value. Unknown bodies discard
    /// this knowledge under the existing conservative replay policy.
    pub value_present: Option<bool>,
    pub ownership_id: Option<u32>,
    pub field_3e0: Option<Option<String>>,
    pub field_3e8: Option<Option<String>>,
    pub field_39c: Option<bool>,
    /// Opaque updates may have changed fields we cannot locate yet.
    pub opaque_updates: usize,
}

impl NativeItemState {
    /// Apply established RxUpdate effects. A received item REPLACES its entire
    /// previous value; common-name/byte omissions do not merge old item fields.
    pub fn apply(&mut self, update: NativeItemBody) {
        if let Some(item) = update.item {
            self.item = Some(item);
            self.value_present = Some(true);
        }
        match update.ownership {
            OwnershipUpdate::Unchanged => (),
            OwnershipUpdate::Sentinel => self.ownership_id = Some(1),
            OwnershipUpdate::Explicit {
                id,
                field_3e0,
                field_3e8,
            } => {
                self.ownership_id = Some(id);
                self.field_3e0 = Some(field_3e0);
                self.field_3e8 = Some(field_3e8);
            }
        }
        self.field_39c = Some(update.field_39c);
    }

    /// Conservative replay policy: an unparsed body/base layer must not leave
    /// stale values presented as known. This is not native gameplay behavior.
    pub fn invalidate_opaque(&mut self) {
        let opaque_updates = self.opaque_updates.saturating_add(1);
        *self = Self {
            opaque_updates,
            ..Self::default()
        };
    }
}

#[cfg(test)]
mod tests;
