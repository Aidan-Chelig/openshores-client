use super::*;

#[test]
fn operator_pose_matches_static_main_and_occupant_jump_tables_for_all_byte_inputs() {
    let map = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/evidence/native-station-operator-pose-map.csv"
    ));
    let rows: Vec<_> = map
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<_> = line.split(',').collect();
            assert_eq!(fields.len(), 4);
            (
                fields[0].parse::<u32>().unwrap(),
                fields[1],
                fields[2]
                    .split(';')
                    .map(|v| v.parse::<u32>().unwrap())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(rows.len(), 60);
    for kind in (0..=255).chain([0x7fff_ffff, 0x8000_0000, u32::MAX]) {
        for occupant in (0..=255).chain([u32::MAX]) {
            for variant in [i32::MIN, -1, 0, 1, 2, 3, 4, 255, i32::MAX] {
                let mut seed = 0_u32;
                let expected = if let Some(row) = rows.get(kind as usize) {
                    assert_eq!(row.0, kind);
                    match row.1 {
                        "constant" => row.2[0],
                        "occupant" => row.2.get(occupant as usize).copied().unwrap_or(0x24),
                        "variant" => {
                            if (0..=3).contains(&variant) {
                                row.2[0] + variant as u32
                            } else {
                                0x24
                            }
                        }
                        "random" => row.2[if variant == 0 { 0 } else { 4 }], // seed0 advances to12345, index0
                        _ => panic!("unknown fixture class"),
                    }
                } else {
                    0x24
                };
                assert_eq!(
                    native_station_operator_pose(kind, occupant, variant, Some(&mut seed)).unwrap(),
                    expected,
                    "type {kind}, occupant {occupant}, variant {variant}"
                );
                assert_eq!(
                    seed,
                    if (0x24..=0x38).contains(&kind) {
                        12345
                    } else {
                        0
                    }
                );
                if !(0x24..=0x38).contains(&kind) {
                    assert_eq!(
                        native_station_operator_pose(kind, occupant, variant, None).unwrap(),
                        expected
                    );
                }
            }
        }
    }
}

#[test]
fn random_pose_uses_shared_unsigned_state_and_both_exact_choice_tables() {
    // Fixed independent arithmetic vectors, including overflow and all four
    // index quadrants. Native multiply/add and f64 scaling constants are saved.
    for (before, after, index) in [
        (0_u32, 0x00003039_u32, 0),
        (1, 0x41c67ea6, 1),
        (2, 0x838ccd13, 2),
        (3, 0xc5531b80, 3),
        (0x12345678, 0x0b719151, 0),
        (0x80000000, 0x80003039, 2),
        (0xffffffff, 0xbe39e1cc, 2),
    ] {
        for kind in 0x24..=0x38 {
            for variant in [i32::MIN, -1, 0, 1, 2, i32::MAX] {
                for occupant in [0, 5, 6, 255, u32::MAX] {
                    let mut seed = before;
                    let pose =
                        native_station_operator_pose(kind, occupant, variant, Some(&mut seed))
                            .unwrap();
                    assert_eq!(seed, after);
                    assert_eq!(pose, (if variant == 0 { 0x4c } else { 0x50 }) + index);
                }
            }
            assert_eq!(
                native_station_operator_pose(kind, 0, 0, None)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::Unsupported
            );
        }
    }
    let mut seed = 0;
    assert_eq!(
        native_station_operator_pose(36, 0, 0, Some(&mut seed)).unwrap(),
        0x4c
    );
    assert_eq!(seed, 12345);
    assert_eq!(
        native_station_operator_pose(36, 0, 0, Some(&mut seed)).unwrap(),
        0x4f
    );
    assert_eq!(seed, 3554416254);
}
