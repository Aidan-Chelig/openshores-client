//! Prefix-only native capture diagnostics; raw records remain unchanged.
use openshores_client::world::native_capture::{CaptureLimits, NativeCapture};
use sha1::{Digest, Sha1};
use std::{fs::File, io::BufReader};

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if args.first().map(String::as_str) != Some("inspect-native-capture") {
        return Ok(false);
    }
    if args.len() != 2 {
        return Err("usage: inspect-native-capture CAPTURE_FILE".into());
    }
    let capture = NativeCapture::read(
        &mut BufReader::new(File::open(&args[1])?),
        CaptureLimits::default(),
    )?;
    println!(
        "{{\"frames\":{},\"payload_bytes\":{}}}",
        capture.frames.len(),
        capture.payload_bytes
    );
    for (index, frame) in capture.frames.iter().enumerate() {
        let (kind, id, position) = if let Some((kind, base)) = frame.prefix() {
            (
                format!("\"{kind:?}\""),
                base.id.to_string(),
                base.position
                    .as_ref()
                    .map(super::positions::json)
                    .unwrap_or("null".into()),
            )
        } else {
            ("null".into(), "null".into(), "null".into())
        };
        println!(
            "{{\"frame\":{},\"opcode\":{},\"kind\":{},\"id\":{},\"position\":{},\"tail_bytes\":{},\"payload_sha1\":\"{:x}\"}}",
            index,
            frame.payload()[0],
            kind,
            id,
            position,
            frame.remainder().len(),
            Sha1::digest(frame.payload())
        );
    }
    Ok(true)
}
