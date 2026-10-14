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
    assert_eq!(capture.frames[0].item_body().unwrap(), None);
    assert_eq!(capture.frames[1].item_body().unwrap(), None);
}

#[test]
fn item_replay_preserves_raw_payloads_and_stops_at_unimplemented_boundaries() {
    for (opcode, base_flags, body, decoded, malformed) in [
        (
            0x16,
            0,
            vec![1, 6, 0, 0, 1, 1, 0x80, 0xde, 0xad],
            true,
            false,
        ),
        (0x16, 0, vec![1, 4, 0xde, 0xad], false, false), // unknown DNA subtype
        (0x16, 0x10, vec![1, 6], false, false),          // effects precede body
        (0x16, 0x40, vec![1, 6], false, false),          // sync precedes body
        (0x05, 0, vec![1, 6], false, false),             // different native kind
        (0xff, 0, vec![1, 6], false, false),
        (0x16, 0, vec![1, 6], false, true), // known truncated common
        (0x16, 0, vec![], false, true),
    ] {
        let mut payload = vec![opcode];
        payload.extend(7_u32.to_be_bytes());
        payload.extend(123_i64.to_be_bytes());
        payload.push(base_flags);
        payload.extend(&body);
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &payload).unwrap();
        let capture =
            NativeCapture::read(&mut Cursor::new(bytes), CaptureLimits::default()).unwrap();
        let frame = &capture.frames[0];
        let result = frame.item_body();
        assert_eq!(result.is_err(), malformed);
        if !malformed {
            assert_eq!(result.unwrap().is_some(), decoded);
        }
        assert_eq!(frame.payload(), payload);
        if opcode != 0xff {
            assert_eq!(frame.remainder(), body);
        }
        if decoded {
            let item = frame.item_body().unwrap().unwrap();
            assert_eq!(item.consumed_bytes, 7);
            assert_eq!(&frame.remainder()[item.consumed_bytes..], [0xde, 0xad]);
            assert_eq!(item.item.unwrap().extension.type_id(), 6);
        }
    }
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
