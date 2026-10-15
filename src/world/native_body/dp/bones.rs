//! DpBody transform writes at body+0x70+i*0x108, i in 0..61.
//! REA ev_416f549167537f850d27a95edbde05f7071c0b975c573dc06a04fb0b97ff0395.
use super::NativeDhDna;
mod multiply;
mod project;
mod skinning;
mod transform;
pub use skinning::NativeDpSkinnedVertex;

const IDENTITY: [f64; 16] = [
    1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
];

/// Only fields actually written by the native identity-reset operations.
/// The intervening native bytes 0x80..0x100 are not initialized by these
/// operations and are deliberately not represented as invented zero matrices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeDpBoneTransform {
    pub matrix_00: [f64; 16],
    pub field_100: u8,
}

impl NativeDpBoneTransform {
    const RESET: Self = Self {
        matrix_00: IDENTITY,
        field_100: 0,
    };
    pub fn identity() -> Self {
        Self::RESET
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpBoneTransforms {
    pub slots: [NativeDpBoneTransform; 61],
}

impl NativeDpBoneTransforms {
    /// Constructor and DpBody::Init reset all slots, regardless of DNA.
    pub fn initialized() -> Self {
        Self {
            slots: [NativeDpBoneTransform::RESET; 61],
        }
    }

    /// Actual InitBoneMatrices selection used on animation updates. Unselected
    /// slots retain both represented fields; this is not an all-slot reset.
    pub fn reset_for_animation(&mut self, dna: &NativeDhDna) {
        let raw = &dna.0;
        let mut reset = |start: usize, count: usize| {
            self.slots[start..start + count].fill(NativeDpBoneTransform::RESET);
        };
        reset(0, usize::from(raw[20] >> 5));
        reset(7, 1);
        reset(59, 1);
        let neck = usize::from((raw[14] & 3) + ((raw[5] >> 1) & 1));
        reset(8, neck);
        if raw[5] & 2 != 0 {
            reset(12, 1);
            reset(60, 1);
            reset(13, neck);
        }
        reset(17, (usize::from(raw[14] >> 6) * 2).max(1));
        for pair in 0..usize::from(raw[1] & 3) {
            reset(23 + pair * 6, 6);
        }
        for pair in 0..usize::from((raw[12] >> 2) & 3) {
            reset(41 + pair * 6, 6);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization_writes_every_slot_to_positive_zero_identity() {
        let bones = NativeDpBoneTransforms::initialized();
        for bone in bones.slots {
            assert_eq!(bone.field_100, 0);
            for (index, value) in bone.matrix_00.iter().enumerate() {
                assert_eq!(
                    value.to_bits(),
                    if index % 5 == 0 {
                        1_f64.to_bits()
                    } else {
                        0_f64.to_bits()
                    }
                );
            }
        }
    }

    #[test]
    fn animation_selection_preserves_inactive_slots_for_all_packed_counts() {
        let sentinel = NativeDpBoneTransform {
            matrix_00: [-9.; 16],
            field_100: 0xa5,
        };
        // Every combination of all six independent selectors (4096 bodies).
        for spine in 0..8 {
            for second_head in 0..2 {
                for neck in 0..4 {
                    for tail in 0..4 {
                        for arms in 0..4 {
                            for legs in 0..4 {
                                let mut dna = NativeDhDna([0; 24]);
                                dna.0[20] = spine << 5;
                                dna.0[5] = second_head << 1;
                                dna.0[14] = neck | tail << 6;
                                dna.0[1] = arms;
                                dna.0[12] = legs << 2;
                                let mut bones = NativeDpBoneTransforms {
                                    slots: [sentinel; 61],
                                };
                                bones.reset_for_animation(&dna);
                                for (index, actual) in bones.slots.iter().enumerate() {
                                    let offset = 0x70 + index * 0x108;
                                    // Independent offset-based expectation from native stores.
                                    let selected = index < usize::from(spine)
                                        || [0x7a8, 0x3d48].contains(&offset)
                                        || (index >= 8
                                            && index < 8 + usize::from(neck + second_head))
                                        || (second_head != 0
                                            && ([0xcd0, 0x3e50].contains(&offset)
                                                || (index >= 13
                                                    && index
                                                        < 13 + usize::from(neck + second_head))))
                                        || (index >= 17
                                            && index < 17 + usize::from((tail * 2).max(1)))
                                        || (index >= 23 && index < 23 + usize::from(arms) * 6)
                                        || (index >= 41 && index < 41 + usize::from(legs) * 6);
                                    assert_eq!(
                                        *actual,
                                        if selected {
                                            NativeDpBoneTransform::RESET
                                        } else {
                                            sentinel
                                        },
                                        "slot {index}, dna {:?}",
                                        dna.0
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
