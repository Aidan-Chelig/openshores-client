//! DpBody::TorsoSegment wrapper and part.0; literal native 6x9 ordinal table.
use super::*;

pub fn native_dp_torso_segment(dna: &NativeDhDna, index: u32) -> u32 {
    let raw = &dna.0;
    let last = u32::from(raw[20] >> 5);
    if index >= last {
        return 0;
    }
    let waist = native_dh_waist_vertex(dna);
    let mut arms = native_dh_arm_vertices(dna);
    let leg_pairs = u32::from((raw[12] >> 2) & 3);
    let legs = if leg_pairs == 0 {
        0
    } else {
        let available = (last + 1).wrapping_sub(arms);
        if available == 0 {
            1
        } else {
            available.min(leg_pairs)
        }
    };
    if raw[1] & 3 != 0 && raw[2] & 6 != 0 && arms != 0 {
        arms -= 1;
    }
    let middle = ((last - waist) >> 1) + waist;
    let top = if index < arms {
        if index == 0 { 1 } else { 2 }
    } else if index < waist || legs == 0 {
        if index == 0 { 0 } else { 3 }
    } else if index == 0 {
        4
    } else if index == waist || legs == 3 && index == middle {
        5
    } else {
        3
    };
    let next = index + 1;
    let tail = raw[14] & 0xc0 != 0;
    let bottom = if next < arms {
        if next == last {
            if tail { 1 } else { 8 }
        } else {
            3
        }
    } else if next < waist {
        4
    } else if next == last {
        if legs == 0 || legs < 2 && waist != next {
            if tail { 7 } else { 0 }
        } else if tail {
            5
        } else {
            2
        }
    } else if legs != 0 && next == waist || legs == 3 && next == middle {
        6
    } else {
        4
    };
    // Native stack table contains the consecutive u32 values1..54.
    top * 9 + bottom + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_literal_segments_and_wrapper_bounds() {
        let mut dna = NativeDhDna([0; 24]);
        assert_eq!(native_dp_torso_segment(&dna, 0), 0);
        dna.0[20] = 0x20;
        assert_eq!(native_dp_torso_segment(&dna, 0), 1);
        assert_eq!(native_dp_torso_segment(&dna, 1), 0);
        dna.0[20] = 0x40;
        assert_eq!(
            (
                native_dp_torso_segment(&dna, 0),
                native_dp_torso_segment(&dna, 1)
            ),
            (5, 28)
        );
        dna.0[1] = 1;
        dna.0[12] = 4;
        assert_eq!(
            (
                native_dp_torso_segment(&dna, 0),
                native_dp_torso_segment(&dna, 1)
            ),
            (14, 30)
        );
        assert_eq!(native_dp_torso_segment(&dna, u32::MAX), 0);
    }
    #[test]
    fn packed_combinations_stay_inside_literal_table_and_ignore_unread_bits() {
        for last in 0..8 {
            for arms in 0..4 {
                for legs in 0..4 {
                    for wing in 0..4 {
                        for tail in 0..4 {
                            let mut dna = NativeDhDna([0; 24]);
                            dna.0[20] = last << 5;
                            dna.0[1] = arms;
                            dna.0[12] = legs << 2;
                            dna.0[2] = wing << 1;
                            dna.0[14] = tail << 6;
                            for index in 0..8 {
                                let actual = native_dp_torso_segment(&dna, index);
                                assert!(if index < u32::from(last) {
                                    (1..=54).contains(&actual)
                                } else {
                                    actual == 0
                                });
                                let mut ignored = dna.clone();
                                ignored.0[16] = 255;
                                ignored.0[19] = 255;
                                assert_eq!(native_dp_torso_segment(&ignored, index), actual);
                            }
                        }
                    }
                }
            }
        }
    }
}
