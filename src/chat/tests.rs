use super::*;
use crate::protocol::QtWriter;
use std::io::Write;

#[test]
fn source_story_controls_are_single_byte_chat_commands() {
    assert_eq!(StoryControl::Close.encode(), [0x7d]);
    assert_eq!(StoryControl::Hint.encode(), [0x7e]);
    assert_eq!(StoryControl::Resend.encode(), [0x7f]);
}

#[test]
fn outbound_chat_has_client_header_and_bounded_close_range_message() {
    let message = Outgoing {
        channel: 2,
        range: Range::Close,
        text: "Hello 🌊".into(),
    };
    let encoded = message.encode(512).unwrap();
    assert_eq!(&encoded[..5], &[0x0a, 0, 0, 0, 0]);
    let mut server_envelope = vec![0x0a];
    server_envelope.extend(&encoded[5..]);
    let Incoming::Message(decoded) = decode_incoming(&server_envelope).unwrap() else {
        panic!("chat")
    };
    assert_eq!(decoded.text.as_deref(), Some("Hello 🌊"));
    assert_eq!(decoded.range, Some((0, 0)));
    assert_eq!(decoded.sender, 512);
    assert!(message.encode(0).is_err());
    assert!(
        Outgoing {
            channel: 22,
            range: Range::Close,
            text: "x".into()
        }
        .encode(512)
        .is_err()
    );
    assert!(
        Outgoing {
            channel: 2,
            range: Range::Close,
            text: "x".repeat(257)
        }
        .encode(512)
        .is_err()
    );
}

#[test]
fn all_source_ranges_encode_both_current_and_maximum_nibbles() {
    let mut range = Range::Close;
    for code in 0..6 {
        let message = Outgoing {
            channel: 2,
            range,
            text: "range test".into(),
        };
        let encoded = message.encode(512).unwrap();
        let mut server_envelope = vec![0x0a];
        server_envelope.extend(&encoded[5..]);
        let Incoming::Message(decoded) = decode_incoming(&server_envelope).unwrap() else {
            panic!("chat")
        };
        assert_eq!(decoded.range, Some((code, code)));
        range = range.next();
    }
    assert_eq!(range, Range::Close);
}

fn envelope(kind: u8) -> Vec<u8> {
    let mut plain = vec![kind];
    plain.extend(fixture(0xe5, false));
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&plain).unwrap();
    let mut compressed = (plain.len() as u32).to_be_bytes().to_vec();
    compressed.extend(encoder.finish().unwrap());
    let mut writer = QtWriter::default();
    writer.write_u8(0x0a);
    writer
        .write_bytes(Some(&crate::protocol::au_crypt(&compressed, 0x6e32)))
        .unwrap();
    writer.into_bytes()
}

#[test]
fn server_envelope_is_exact_and_unknown_types_stay_opaque() {
    let bytes = envelope(0x29);
    let Incoming::Message(message) = decode_incoming(&bytes).unwrap() else {
        panic!("expected chat")
    };
    assert_eq!(message.text.as_deref(), Some("Hello 🌊"));
    for end in 0..bytes.len() {
        assert!(decode_incoming(&bytes[..end]).is_err());
    }
    assert!(matches!(
        decode_incoming(&envelope(0xfe)).unwrap(),
        Incoming::Opaque { kind: 0xfe, .. }
    ));
    let mut trailing = bytes;
    trailing.push(0);
    assert!(decode_incoming(&trailing).is_err());
}

fn fixture(flags: u8, named: bool) -> Vec<u8> {
    let mut writer = QtWriter::default();
    writer.write_u8(if named { 255 } else { 2 });
    if named {
        writer.write_string(Some("Local custom channel")).unwrap();
    }
    writer.write_u8(flags);
    writer.write_u32(512);
    writer.write_string(Some("Rust Explorer")).unwrap();
    writer.write_u32(0);
    if flags & 0x40 != 0 {
        writer.write_u8(0xa3);
    }
    writer.write_u8(1);
    if flags & 0x80 != 0 {
        writer.write_u32((-7i32) as u32);
    }
    writer.write_string(Some("Hello 🌊")).unwrap();
    writer.into_bytes()
}

#[test]
fn source_fields_preserve_flags_optional_ranges_and_unicode() {
    for flags in [0, 0x40, 0x80, 0xe5] {
        for named in [false, true] {
            let bytes = fixture(flags, named);
            let message = Message::decode(&bytes).unwrap();
            assert_eq!(message.flags, flags);
            assert_eq!(message.sender, 512);
            assert_eq!(message.text.as_deref(), Some("Hello 🌊"));
            assert_eq!(message.range, (flags & 0x40 != 0).then_some((3, 10)));
            assert_eq!(message.auxiliary, (flags & 0x80 != 0).then_some(-7));
            assert_eq!(
                message.channel,
                if named {
                    Channel::Named(Some("Local custom channel".into()))
                } else {
                    Channel::Indexed(2)
                }
            );
            for end in 0..bytes.len() {
                assert!(Message::decode(&bytes[..end]).is_err());
            }
            let mut trailing = bytes;
            trailing.push(0);
            assert!(Message::decode(&trailing).is_err());
        }
    }
}

#[test]
fn unknown_channels_and_input_budgets_are_explicit() {
    let mut bytes = fixture(0, false);
    bytes[0] = 100;
    assert_eq!(
        Message::decode(&bytes).unwrap().channel,
        Channel::Indexed(100)
    );
    bytes[0] = 254;
    assert!(Message::decode(&bytes).is_err());
    assert!(Message::decode(&vec![0; 65537]).is_err());
}

#[test]
fn offered_choice_fields_and_bare_reply_are_exact() {
    let body = fixture(0, false);
    let text_bytes = "Hello 🌊".encode_utf16().count() * 2;
    let mut choice = body[..body.len() - text_bytes - 4].to_vec();
    let mut tail = QtWriter::default();
    tail.write_u8(3);
    tail.write_u32(1);
    tail.write_string(Some("Go on")).unwrap();
    tail.write_u32(700);
    choice.extend(tail.into_bytes());
    let decoded = Choice::decode(&choice).unwrap();
    assert_eq!(decoded.index, 3);
    assert_eq!(decoded.instance, 1);
    assert_eq!(decoded.offered_to, 700);
    assert_eq!(decoded.prompt.as_deref(), Some("Go on"));
    assert_eq!(
        decoded.reply().encode().unwrap(),
        [0x7c, 0, 0, 0, 1, 3, 0, 0, 2, 0]
    );
    for end in 0..choice.len() {
        assert!(Choice::decode(&choice[..end]).is_err());
    }
    assert!(
        Reply {
            instance: 1,
            index: 128,
            offered: 512
        }
        .encode()
        .is_err()
    );
}

#[test]
fn hail_request_matches_source_parent_and_tail_fields() {
    let encoded = Hail::encode(512, "Rust Explorer").unwrap();
    let mut envelope = QtReader::new(&encoded[5..]);
    let cipher = envelope.read_bytes().unwrap().unwrap();
    let plain = crate::har::qdecompress(&crate::protocol::au_crypt(cipher, 0x6e32), 65536).unwrap();
    assert_eq!(plain[0], 0x3f);
    let mut input = QtReader::new(&plain[1..]);
    let parent = header(&mut input).unwrap();
    assert_eq!(parent.channel, Channel::Indexed(2));
    assert_eq!(parent.sender, 512);
    assert_eq!(parent.sender_name.as_deref(), Some("Rust Explorer"));
    assert_eq!(parent.range, Some((0, 0)));
    assert_eq!(parent.scope, 15);
    assert_eq!(input.read_u32().unwrap(), 0);
    assert_eq!(input.read_u8().unwrap(), 0);
    assert_eq!(input.remaining(), 0);
    assert!(Hail::encode(0, "").is_err());
}

#[test]
fn narration_and_narrated_choice_keep_instance_and_distinct_tail_layouts() {
    let bytes = fixture(0, false);
    let header_len = bytes.len() - 4 - "Hello 🌊".encode_utf16().count() * 2;
    let mut tail = QtWriter::default();
    tail.write_u32(0x1001);
    tail.write_string(Some("Story 🌊")).unwrap();
    let tail = tail.into_bytes();
    let body = [bytes[..header_len].to_vec(), tail.clone()].concat();
    for kind in [0x2d, 0x4a, 0x4e] {
        let story = StoryText::decode(kind, &body).unwrap();
        assert_eq!(story.instance, 0x1001);
        assert_eq!(story.header.text.as_deref(), Some("Story 🌊"));
        for end in 0..body.len() {
            assert!(StoryText::decode(kind, &body[..end]).is_err());
        }
    }
    let choice_body = [bytes[..header_len].to_vec(), vec![2], tail].concat();
    let choice = Choice::decode_narration(&choice_body).unwrap();
    assert_eq!(choice.instance, 0x1001);
    assert_eq!(choice.index, 2);
    assert_eq!(choice.offered_to, choice.header.target);
    for end in 0..choice_body.len() {
        assert!(Choice::decode_narration(&choice_body[..end]).is_err());
    }
    assert!(Choice::decode(&choice_body).is_err());
    assert_eq!(
        Reply {
            instance: 0x1001,
            index: 2,
            offered: 0
        }
        .encode()
        .unwrap(),
        [0x7c, 0, 0, 0x10, 1, 2, 0, 0, 0, 0]
    );
}
