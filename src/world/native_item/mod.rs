//! Patched-client item readers, separate from the reference server profile.
//! Supported subtype readers are bound to native factory/vtable evidence.
mod atom;
mod body;
pub use atom::{
    NativeItemAtom, NativeItemCandidateReceiveContext, NativeItemObjects,
    NativeItemOwnedReceiveContext,
};
mod common;
mod construction;
pub use construction::NativeItemInitialization;
mod state;
mod value;
mod weapon;
pub use body::{NativeItemBody, OwnershipUpdate};
pub use common::NativeItemCommon;
pub use state::NativeItemState;
pub use value::{NativeItemExtension, NativeItemValue};
pub use weapon::{NativeWeapon, NativeWeaponFields, NativeWeaponGroup};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod value_tests;
