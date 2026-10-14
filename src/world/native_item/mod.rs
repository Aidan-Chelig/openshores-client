//! Patched-client item readers, separate from the reference server profile.
//! Supported subtype readers are bound to native factory/vtable evidence.
mod body;
mod common;
mod value;
mod weapon;
pub use body::{NativeItemBody, OwnershipUpdate};
pub use common::NativeItemCommon;
pub use value::{NativeItemExtension, NativeItemValue};
pub use weapon::{NativeWeapon, NativeWeaponFields, NativeWeaponGroup};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod value_tests;
