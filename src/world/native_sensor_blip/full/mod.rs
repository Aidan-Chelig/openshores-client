//! Fresh +158 full bodies, not the +190 minimal list representation.
use super::{read_float, read_prefix};
use crate::{atom_position::FloatEncoding, protocol::QtReader, world::native_dna::NativeSpecies};
use std::io;
mod types;
pub use types::*;
mod read;

#[cfg(test)]
mod tests;
