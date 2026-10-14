use super::*;
use crate::protocol::write_frame;
use std::io::Cursor;

fn known() -> Vec<u8> {
    let mut bytes = vec![0x16];
    bytes.extend(7_u32.to_be_bytes());
    bytes.extend(123_i64.to_be_bytes());
    bytes.extend([0x10, 0xde, 0xad]); // opaque effect layer, not an item body
    bytes
}

#[test]
fn native_capture_keeps_unknown_packets_and_known_opaque_layers_exactly() {
    let mut bytes = Vec::new();
    let unknown = [0xff, 1, 2, 3];
    write_frame(&mut bytes, &unknown).unwrap();
    write_frame(&mut bytes, &known()).unwrap();
    let capture = NativeCapture::read(&mut Cursor::new(bytes), CaptureLimits::default()).unwrap();
    assert_eq!(capture.frames.len(), 2);
    assert_eq!(capture.payload_bytes, unknown.len() + known().len());
    assert!(capture.frames[0].prefix().is_none());
    assert_eq!(capture.frames[0].remainder(), unknown);
    let (kind, base) = capture.frames[1].prefix().unwrap();
    assert_eq!(*kind, NativeAtomKind::Item);
    assert_eq!(
        base.tail_kind,
        super::super::native_prefix::TailKind::EffectsAndFollowing
    );
    assert_eq!(capture.frames[1].remainder(), [0xde, 0xad]);
    assert_eq!(capture.frames[1].payload(), known());
}

#[test]
fn frame_count_per_frame_and_aggregate_budgets_are_enforced() {
    let mut bytes = Vec::new();
    write_frame(&mut bytes, &known()).unwrap();
    write_frame(&mut bytes, &known()).unwrap();
    for limits in [
        CaptureLimits {
            frames: 1,
            ..Default::default()
        },
        CaptureLimits {
            frame_bytes: known().len() - 1,
            ..Default::default()
        },
        CaptureLimits {
            payload_bytes: known().len() * 2 - 1,
            ..Default::default()
        },
    ] {
        assert!(NativeCapture::read(&mut Cursor::new(&bytes), limits).is_err());
    }
    assert!(
        NativeCapture::read(
            &mut Cursor::new([]),
            CaptureLimits {
                frames: usize::MAX,
                ..Default::default()
            }
        )
        .is_err()
    );
}

#[test]
fn empty_capture_and_truncated_frames_or_known_atoms_are_distinguished() {
    assert!(
        NativeCapture::read(&mut Cursor::new([]), CaptureLimits::default())
            .unwrap()
            .frames
            .is_empty()
    );
    let mut bytes = Vec::new();
    write_frame(&mut bytes, &known()).unwrap();
    for length in 1..bytes.len() {
        assert!(
            NativeCapture::read(&mut Cursor::new(&bytes[..length]), CaptureLimits::default())
                .is_err()
        );
    }
    let mut bytes = Vec::new();
    write_frame(&mut bytes, &[0x16]).unwrap();
    assert!(NativeCapture::read(&mut Cursor::new(bytes), CaptureLimits::default()).is_err());
}
