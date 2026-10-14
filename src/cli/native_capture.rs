//! Native capture diagnostics; raw records remain unchanged.
use openshores_client::world::{
    native_capture::{CaptureLimits, NativeCapture},
    native_item::{NativeItemBody, OwnershipUpdate},
};
use sha1::{Digest, Sha1};
use std::{fs::File, io::BufReader};

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    let command = args.first().map(String::as_str);
    if !matches!(
        command,
        Some("inspect-native-capture" | "inspect-native-item-capture")
    ) {
        return Ok(false);
    }
    if args.len() != 2 {
        return Err(
            "usage: inspect-native-capture|inspect-native-item-capture CAPTURE_FILE".into(),
        );
    }
    let capture = NativeCapture::read(
        &mut BufReader::new(File::open(&args[1])?),
        CaptureLimits::default(),
    )?;
    // Validate every requested body before emitting any output. Keep summaries,
    // not a second collection of potentially large decoded strings.
    let items = if command == Some("inspect-native-item-capture") {
        Some(
            capture
                .frames
                .iter()
                .map(|frame| {
                    frame.item_body().map(|body| {
                        body.as_ref()
                            .map(|body| summary(body, frame.remainder().len()))
                            .unwrap_or("null".into())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        )
    } else {
        None
    };
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
            "{{\"frame\":{},\"opcode\":{},\"kind\":{},\"id\":{},\"position\":{},\"tail_bytes\":{},\"payload_sha1\":\"{:x}\"{}}}",
            index,
            frame.payload()[0],
            kind,
            id,
            position,
            frame.remainder().len(),
            Sha1::digest(frame.payload()),
            items
                .as_ref()
                .map(|items| format!(",\"item\":{}", items[index]))
                .unwrap_or_default()
        );
    }
    Ok(true)
}

fn summary(body: &NativeItemBody, tail_bytes: usize) -> String {
    let value = body
        .item
        .as_ref()
        .map(|item| {
            format!(
                "{{\"type_id\":{},\"commodity\":{},\"common_flags\":{}}}",
                item.extension.type_id(),
                item.common.commodity,
                item.common.flags
            )
        })
        .unwrap_or("null".into());
    let ownership = match &body.ownership {
        OwnershipUpdate::Unchanged => "{\"kind\":\"unchanged\"}".into(),
        OwnershipUpdate::Sentinel => "{\"kind\":\"sentinel\",\"id\":1}".into(),
        OwnershipUpdate::Explicit { id, .. } => format!("{{\"kind\":\"explicit\",\"id\":{id}}}"),
    };
    format!(
        "{{\"flags\":{},\"value\":{},\"ownership\":{},\"field_39c\":{},\"consumed_bytes\":{},\"remaining_bytes\":{}}}",
        body.flags,
        value,
        ownership,
        body.field_39c,
        body.consumed_bytes,
        tail_bytes - body.consumed_bytes
    )
}
