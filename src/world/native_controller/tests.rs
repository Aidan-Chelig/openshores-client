use super::*;
fn position_bytes() -> Vec<u8> {
    let mut bytes = u32::MAX.to_be_bytes().to_vec();
    for value in [12345678.125_f64, -2.5, 3.75, 0.1, 0.2, 0.3] {
        bytes.extend(value.to_be_bytes());
    }
    bytes
}
#[test]
fn concrete_factory_bodies_have_exact_boundaries_and_atomic_truncation() {
    for kind in 0..=4 {
        let mut bytes = match kind {
            0 | 3 => position_bytes(),
            1 => [1_f64, 2., 3.]
                .into_iter()
                .flat_map(f64::to_be_bytes)
                .collect(),
            2 => vec![],
            4 => {
                let mut bytes = 1.25_f32.to_be_bytes().to_vec();
                bytes.extend([255, 128]);
                bytes.extend(i16::MIN.to_be_bytes());
                bytes
            }
            _ => unreachable!(),
        };
        if kind == 3 {
            bytes.push(0);
        }
        let length = bytes.len();
        bytes.extend([0xde, 0xad]);
        let mut input = QtReader::new(&bytes);
        let value = NativeController::read(kind, &mut input).unwrap().unwrap();
        assert_eq!(input.remaining_bytes(), &[0xde, 0xad]);
        match value {
            NativeController::AtomPosition(p)
            | NativeController::AtomPositionVector { position: p, .. } => {
                assert_eq!(p.parent, u32::MAX);
                assert_eq!(p.position[0], 12345678.125);
                assert_eq!(p.rotation, [0.1, 0.2, 0.3]);
            }
            NativeController::Pose {
                field_0c,
                field_10,
                field_12,
                ..
            } => {
                assert_eq!(field_0c, 255);
                assert_eq!(field_10, 128);
                assert_eq!(field_12, i16::MIN);
            }
            _ => (),
        }
        for truncated in 0..length {
            let mut input = QtReader::new(&bytes[..truncated]);
            assert!(NativeController::read(kind, &mut input).is_err());
            assert_eq!(input.remaining(), truncated);
        }
    }
}
#[test]
fn all_atom_position_vector_flags_preserve_widths_and_omission() {
    for flags in 0..=255 {
        let mut bytes = position_bytes();
        bytes.push(flags);
        if flags & 2 != 0 {
            for value in [4_f64, 5., 6.] {
                bytes.extend(value.to_be_bytes());
            }
        }
        bytes.push(0xde);
        let mut input = QtReader::new(&bytes);
        let NativeController::AtomPositionVector {
            flags: actual,
            field_118,
            vector_120,
            ..
        } = NativeController::read(3, &mut input).unwrap().unwrap()
        else {
            panic!()
        };
        assert_eq!(actual, flags);
        assert_eq!(field_118, flags & 1 != 0);
        assert_eq!(
            vector_120,
            if flags & 2 != 0 {
                [4., 5., 6.]
            } else {
                [0.; 3]
            }
        );
        assert_eq!(input.remaining_bytes(), &[0xde]);
    }
    for kind in 5..=255 {
        let mut input = QtReader::new(&[0xde]);
        assert!(NativeController::read(kind, &mut input).unwrap().is_none());
        assert_eq!(input.remaining_bytes(), &[0xde]);
    }
}
