//! Established DaItem() initialization, not packet-driven object creation.
use super::NativeItemState;
use crate::world::native_atom::NativeBaseAtomInitialization;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeItemInitialization {
    pub base: NativeBaseAtomInitialization,
    pub state: NativeItemState,
}
impl NativeItemInitialization {
    /// Recovered base/owned-item subset. Constructor starts with no AuItem
    /// allocation; its subtype/common defaults are not invented. Shared-null
    /// ownership strings are known Qt-null, distinct from an unobserved snapshot.
    /// Unmodeled item timers/Qt locks and receiver status changes stay separate.
    pub fn from_default_item_constructor() -> Self {
        Self {
            base: NativeBaseAtomInitialization::from_default_base_constructor(),
            state: NativeItemState {
                item: None,
                value_present: Some(false),
                ownership_id: Some(0),
                field_3e0: Some(None),
                field_3e8: Some(None),
                field_39c: Some(false),
                opaque_updates: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests;
