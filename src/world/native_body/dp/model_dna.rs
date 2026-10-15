//! Au3dsGLSLDNA constructor, DhDNA::CreateModelDNA and native comparison order.
use super::NativeDhDna;
use std::cmp::Ordering;

// In increasing destination-field order: (DNA byte, shift, mask). DhDNA's
// native bytes begin at+8; indices here refer to its decoded 24-byte storage.
const LAYOUT: [(usize, u8, u8); 47] = [
    (13, 3, 31),
    (20, 5, 7),
    (14, 0, 3),
    (14, 6, 3),
    (5, 1, 1),
    (1, 0, 3),
    (12, 2, 3),
    (2, 0, 7),
    (12, 4, 3),
    (19, 3, 31),
    (20, 0, 31),
    (14, 4, 3),
    (18, 2, 3),
    (19, 0, 7),
    (16, 0, 7),
    (11, 0, 31),
    (11, 5, 7),
    (10, 0, 31),
    (10, 5, 7),
    (22, 5, 7),
    (21, 0, 31),
    (22, 0, 31),
    (6, 3, 31),
    (8, 3, 31),
    (15, 3, 31),
    (16, 3, 31),
    (2, 3, 31),
    (3, 3, 31),
    (13, 0, 7),
    (12, 6, 3),
    (15, 0, 7),
    (14, 2, 3),
    (7, 6, 3),
    (12, 0, 3),
    (7, 2, 3),
    (8, 0, 7),
    (4, 4, 3),
    (6, 0, 7),
    (7, 4, 3),
    (18, 4, 3),
    (1, 2, 3),
    (4, 2, 3),
    (1, 4, 15),
    (18, 6, 3),
    (21, 5, 7),
    (3, 0, 7),
    (4, 0, 3),
];

// Lexicographic unsigned-u32 comparisons follow native offsets, NOT memory
// order or memcmp. All48 fields participate, including the constructor-only bc.
const ORDER: [usize; 48] = [
    0x00, 0x24, 0x28, 0x3c, 0x44, 0x50, 0x54, 0x58, 0x5c, 0x60, 0x64, 0x68, 0x6c, 0xa8, 0xb0, 0x34,
    0xb4, 0x04, 0x1c, 0x38, 0x40, 0x48, 0x4c, 0x70, 0x8c, 0x94, 0x78, 0x08, 0x0c, 0x14, 0x18, 0x20,
    0x2c, 0x30, 0x74, 0x7c, 0x80, 0x84, 0x88, 0x90, 0x98, 0x9c, 0xa0, 0xa4, 0xac, 0xb8, 0x10, 0xbc,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeDpModelDna {
    fields: [u32; 48],
}

impl Default for NativeDpModelDna {
    fn default() -> Self {
        Self { fields: [0; 48] }
    }
}

impl NativeDpModelDna {
    pub fn from_dhdna(dna: &NativeDhDna) -> Self {
        let mut key = Self::default();
        key.apply_dhdna(dna);
        key
    }

    pub fn from_fields(fields: [u32; 48]) -> Self {
        Self { fields }
    }
    pub fn fields(&self) -> &[u32; 48] {
        &self.fields
    }

    /// Exact47 writes; fieldbc is retained, not cleared by CreateModelDNA.
    pub fn apply_dhdna(&mut self, dna: &NativeDhDna) {
        for (target, (byte, shift, mask)) in self.fields.iter_mut().zip(LAYOUT) {
            *target = u32::from((dna.0[byte] >> shift) & mask);
        }
    }

    pub(super) fn force_hand_field_bc(&mut self) {
        self.fields[0xbc / 4] = 1;
    }
}

impl Ord for NativeDpModelDna {
    fn cmp(&self, other: &Self) -> Ordering {
        for offset in ORDER {
            let order = self.fields[offset / 4].cmp(&other.fields[offset / 4]);
            if order != Ordering::Equal {
                return order;
            }
        }
        Ordering::Equal
    }
}
impl PartialOrd for NativeDpModelDna {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests;
