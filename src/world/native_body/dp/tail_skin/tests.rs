use super::*;

fn part() -> NativeDpPartState {
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: NativeDpPartKind::Tail,
            index: 0,
            dna: NativeDhDna([0; 24]),
        },
        connect_30: [1., 2., 3.],
        joints: NativeDpPartJoints::None,
    }
}

#[test]
fn all_prefix_type_segment_combinations_follow_native_tail_names() {
    let owner = NativeDhDna([0; 24]);
    for prefix in 0..8 {
        for kind in 0..4 {
            for segments in 0..4 {
                let mut part = part();
                part.allocation.dna.0[16] = prefix | 0xf8;
                part.allocation.dna.0[14] = (segments << 6) | (kind << 4) | 15;
                let request = part.tail_skin_geometry(&owner, [1., 2., 3., 4.]).unwrap();
                if kind == 0 {
                    assert!(request.is_none());
                    continue;
                }
                let request = request.unwrap();
                let stem = if segments == 0 {
                    "Tip".to_owned()
                } else {
                    format!("Tail{}", segments * 2 - 1)
                };
                assert_eq!(request.resource, format!(":data/{prefix}{stem}{kind}.3dsN"));
                assert_eq!(request.bones, [17; 3]);
                assert_eq!(request.geometry.fields_40, [1., 2., 3.]);
            }
        }
    }
}

#[test]
fn native_tail_float_divisions_and_owner_scale_are_not_limb_formulas() {
    let mut owner = NativeDhDna([0; 24]);
    owner.0[13] = 0xf8;
    for length_bits in 0..32 {
        for width_bits in 0..32 {
            let mut part = part();
            part.allocation.dna.0[14] = 0x10;
            part.allocation.dna.0[19] = length_bits << 3;
            part.allocation.dna.0[20] = width_bits;
            let request = part
                .tail_skin_geometry(&owner, [1., 2., 3., 4.])
                .unwrap()
                .unwrap();
            let length = f32::from(length_bits) * 0.0625 + 0.5;
            let width = f32::from(width_bits) * 0.0625 * 0.25 * length;
            assert_eq!(
                request.geometry.fields_58,
                [
                    1.,
                    1.,
                    f64::from(length / 1.5_f32),
                    f64::from((width + length / 3_f32) / 0.875_f32),
                    1.
                ]
            );
            assert_eq!(
                request.geometry.fields_00,
                [
                    1.,
                    f64::from(native_dp_size_factor(&owner) / 0.84_f32),
                    1.,
                    2.,
                    3.,
                    4.
                ]
            );
        }
    }
    assert_ne!(f64::from(0.5_f32 / 1.5_f32), 0.5_f64 / 1.5);
}

#[test]
fn wrong_kind_and_incomplete_owner_fail_instead_of_publishing_geometry() {
    let owner = NativeDhDna([0; 24]);
    let mut wrong = part();
    wrong.allocation.kind = NativeDpPartKind::Torso;
    assert_eq!(
        wrong
            .tail_skin_geometry(&owner, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(
        NativeDpInitialization::begin(&owner)
            .tail_skin_geometry_request(0, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
}
