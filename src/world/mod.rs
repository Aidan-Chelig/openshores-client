//! Scene atom prefixes and sparse state, independent of rendering/UI frameworks.
//! Source-server and patched-native profiles remain explicitly separate.
mod atom;
mod capture;
mod state;
pub use atom::{AtomKind, AtomUpdate, Transform, WorldIdentity, decode_atom};
pub use capture::read_capture;
pub use state::{Entity, WorldState};
use std::io;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub mod native_atom;
pub mod native_capture;
pub mod native_condition;
pub mod native_controller;
pub mod native_dispatch;
pub mod native_effect;
pub mod native_item;
pub mod native_position_updates;
pub mod native_prefix;
pub mod native_runaways;
pub mod native_signals;
pub mod native_subatoms;
pub mod native_tick_tock;
mod placement;
#[cfg(test)]
mod tests;
