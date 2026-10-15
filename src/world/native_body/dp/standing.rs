//! CGHeightStanding, including calls discarded by some ankle branches.
use super::{NativeDhDna, NativeDpDimensions, NativeDpFootCache, native_dh_has_ankle_bend};
use std::io;

impl NativeDpFootCache<'_> {
    pub fn standing_height(
        &mut self,
        dna: &NativeDhDna,
        dimensions: &NativeDpDimensions,
        field_4068: i32,
        include_foot_height: bool,
    ) -> io::Result<f32> {
        calculate(dna, dimensions, include_foot_height, |height| {
            if height {
                self.foot_height(dna, field_4068)
            } else {
                self.foot_length(dna, field_4068, false)
            }
        })
    }
}

fn calculate(
    dna: &NativeDhDna,
    dimensions: &NativeDpDimensions,
    include: bool,
    mut measure: impl FnMut(bool) -> io::Result<f32>,
) -> io::Result<f32> {
    let d = dimensions.fields_4080_through_40b8();
    let mut floor = d[11] * 0.5;
    if dna.0[12] & 12 == 0 {
        return Ok(if floor > 0. { floor } else { 0. });
    }
    // Required even when the ankle branch does not use its returned length.
    let length = measure(false)?;
    let segments = d[7] + d[0];
    let class = (dna.0[1] >> 2) & 3;
    let ankle = native_dh_has_ankle_bend(dna);
    let factor = match class {
        0 => 0.1_f32,
        1 => 0.4,
        2 => 0.6,
        _ => {
            if ankle {
                0.99
            } else {
                0.9
            }
        }
    };
    let mut height = if ankle {
        let scaled = segments * factor;
        if class < 2 { length + scaled } else { scaled }
    } else {
        (length + segments) * factor
    };
    if include && ankle {
        height += measure(true)?;
        floor -= measure(true)?;
    }
    // Native MAXSS selects the second operand on equality or unordered input.
    Ok(if floor > height { floor } else { height })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_standing_classes_preserve_required_measurement_calls() {
        for class in 0..4 {
            for ankle in [false, true] {
                for include in [false, true] {
                    let mut dna = NativeDhDna([0; 24]);
                    dna.0[1] = class << 2;
                    dna.0[10] = if ankle { 0 } else { 0x80 };
                    dna.0[12] = 4;
                    let dims = NativeDpDimensions::from_dna(&dna);
                    let mut calls = vec![];
                    let actual = calculate(&dna, &dims, include, |height| {
                        calls.push(height);
                        Ok(if height { 0.125 } else { 0.75 })
                    })
                    .unwrap();
                    assert_eq!(
                        calls,
                        if include && ankle {
                            vec![false, true, true]
                        } else {
                            vec![false]
                        }
                    );
                    let d = dims.fields_4080_through_40b8();
                    let factor = match (class, ankle) {
                        (0, _) => 0.1,
                        (1, _) => 0.4,
                        (2, _) => 0.6,
                        (_, true) => 0.99,
                        _ => 0.9,
                    };
                    let mut expected = (d[7] + d[0]) * factor;
                    if !ankle {
                        expected = (0.75 + (d[7] + d[0])) * factor;
                    } else if class < 2 {
                        expected += 0.75;
                    }
                    if include && ankle {
                        expected += 0.125;
                    }
                    let floor = d[11] * 0.5 - if include && ankle { 0.125 } else { 0. };
                    assert_eq!(actual.to_bits(), expected.max(floor).to_bits());
                }
            }
        }
    }
    #[test]
    fn no_legs_never_request_foot_resources_but_legged_bodies_propagate_errors() {
        let mut dna = NativeDhDna([0; 24]);
        let dims = NativeDpDimensions::from_dna(&dna);
        assert_eq!(
            calculate(&dna, &dims, true, |_| panic!()).unwrap(),
            dims.fields_4080_through_40b8()[11] * 0.5
        );
        dna.0[12] = 4;
        assert_eq!(
            calculate(&dna, &dims, false, |_| Err(io::Error::new(
                io::ErrorKind::NotFound,
                "foot"
            )))
            .unwrap_err()
            .kind(),
            io::ErrorKind::NotFound
        );
    }
}
