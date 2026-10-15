//! Exact packed DNA selectors used by native body initialization.
use super::NativeDhDna;

const SIZE_BITS: [u32; 32] = [
    0x3e9fdc59, 0x3eb99b91, 0x3ed29dc7, 0x3eeb03e2, 0x3f0175c0, 0x3f0d3847, 0x3f18d680, 0x3f245cdd,
    0x3f2fd75e, 0x3f3b51df, 0x3f46d83c, 0x3f527675, 0x3f5e38fc, 0x3f6a2ccb, 0x3f765fd9, 0x3f817050,
    0x3f87e037, 0x3f8e887b, 0x3f9572ef, 0x3f9cabb4, 0x3fa43f92, 0x3fac3dee, 0x3fb4b8d0, 0x3fbdc5d6,
    0x3fc77de9, 0x3fd200d2, 0x3fdd73eb, 0x3fea06b8, 0x3ff7f4dc, 0x4003c52e, 0x400c9596, 0x4016adac,
];

/// DpBody::SizeFactor: literal table, not a fitted/exponentiated size curve.
pub fn native_dp_size_factor(dna: &NativeDhDna) -> f32 {
    f32::from_bits(SIZE_BITS[usize::from(dna.0[13] >> 3)])
}

pub fn native_dh_arm_vertices(dna: &NativeDhDna) -> u32 {
    let arms = u32::from(dna.0[1] & 3);
    if arms == 0 {
        return 0;
    }
    let last = u32::from(dna.0[20] >> 5);
    if last == 0 {
        return 1;
    }
    let legs = u32::from((dna.0[12] >> 2) & 3);
    let mut available = last + 1;
    if legs != 0 {
        let mut divided = available * arms / (legs + arms);
        if available <= divided {
            divided = last;
        }
        available = divided.max(1);
    }
    available.min(arms)
}

pub fn native_dh_waist_vertex(dna: &NativeDhDna) -> u32 {
    let last = u32::from(dna.0[20] >> 5);
    let legs = u32::from((dna.0[12] >> 2) & 3);
    if legs == 1 {
        return last;
    }
    let arms = u32::from(dna.0[1] & 3);
    if arms == 0 {
        return 0;
    }
    let mut value = native_dh_arm_vertices(dna);
    if legs == 0 && !matches!(dna.0[11] & 0xe0, 0x60 | 0xc0) && (dna.0[2] & 6 == 0 || arms > 1) {
        value -= 1;
    }
    value.min(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_table_ignores_three_low_bits() {
        for (index, bits) in SIZE_BITS.into_iter().enumerate() {
            for low in 0..8 {
                let mut dna = NativeDhDna([0; 24]);
                dna.0[13] = (index as u8) << 3 | low;
                assert_eq!(native_dp_size_factor(&dna).to_bits(), bits);
            }
        }
    }

    #[test]
    fn packed_vertex_selectors_cover_all_relevant_bit_combinations() {
        for arms in 0..4 {
            for legs in 0..4 {
                for last in 0..8 {
                    for class in 0..8 {
                        for wings in 0..4 {
                            let mut dna = NativeDhDna([0; 24]);
                            dna.0[1] = arms;
                            dna.0[12] = legs << 2;
                            dna.0[20] = last << 5;
                            dna.0[11] = class << 5;
                            dna.0[2] = wings << 1;
                            let av = native_dh_arm_vertices(&dna);
                            let wv = native_dh_waist_vertex(&dna);
                            assert!(av <= u32::from(arms));
                            assert_eq!(av == 0, arms == 0);
                            assert!(wv <= u32::from(last));
                            if legs == 1 {
                                assert_eq!(wv, u32::from(last));
                            } else if arms == 0 {
                                assert_eq!(wv, 0);
                            } else if legs != 0 {
                                assert_eq!(wv, av.min(u32::from(last)));
                            } else {
                                let decrement = !matches!(class, 3 | 6) && (wings == 0 || arms > 1);
                                assert_eq!(wv, (av - u32::from(decrement)).min(u32::from(last)));
                            }
                        }
                    }
                }
            }
        }
    }
}
