//! Explicit native-prefix diagnostics, never a connected command.
use super::positions;
use openshores_client::{
    protocol::QtReader,
    world::native_prefix::{NativeBasePrefix, TailKind},
};

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if args.first().map(String::as_str) != Some("inspect-native-base") {
        return Ok(false);
    }
    if args.len() != 3 {
        return Err(
            "usage: inspect-native-base type-widths|single|double HEX_FIELDS_AFTER_OPCODE".into(),
        );
    }
    let (encoding, bytes) = positions::input(args, 4096)?;
    let mut input = QtReader::new(&bytes);
    let prefix = NativeBasePrefix::read(&mut input, encoding)?;
    let position = prefix
        .position
        .as_ref()
        .map(positions::json)
        .unwrap_or("null".into());
    let created = prefix
        .created_ms
        .map(|v| v.to_string())
        .unwrap_or("null".into());
    let modified = prefix
        .modified_ms
        .map(|v| v.to_string())
        .unwrap_or("null".into());
    let tail_kind = match prefix.tail_kind {
        TailKind::TypeSpecific => "type-specific",
        TailKind::EffectsAndFollowing => "effects-and-following",
        TailKind::SyncControllersAndFollowing => "sync-controllers-and-following",
    };
    let tail: String = bytes[prefix.consumed_bytes..]
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect();
    println!(
        "{{\"id\":{},\"timestamp_ms\":{},\"flags\":{},\"position\":{},\"created_ms\":{},\"modified_ms\":{},\"in_motion\":{},\"consumed_bytes\":{},\"tail_kind\":\"{}\",\"tail_hex\":\"{}\"}}",
        prefix.id,
        prefix.timestamp_ms,
        prefix.flags,
        position,
        created,
        modified,
        prefix.in_motion,
        prefix.consumed_bytes,
        tail_kind,
        tail
    );
    Ok(true)
}
