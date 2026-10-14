//! Explicit native-prefix diagnostics, never a connected command.
use super::positions;
use openshores_client::{
    protocol::QtReader,
    world::native_atom::NativeAtomPrefix,
    world::native_prefix::{NativeBasePrefix, TailKind},
};

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if args.first().map(String::as_str) == Some("inspect-native-atom") {
        if args.len() != 2 {
            return Err("usage: inspect-native-atom HEX_UNFRAMED_PAYLOAD".into());
        }
        let input_args = vec![args[0].clone(), "type-widths".into(), args[1].clone()];
        let (_, bytes) = positions::input(&input_args, 4096)?;
        match NativeAtomPrefix::decode(&bytes)? {
            Some(prefix) => println!(
                "{{\"kind\":\"{:?}\",\"can_render_environment\":{},\"is_instance_universe\":{},\"in_star_field\":{},\"base\":{}}}",
                prefix.kind,
                prefix.kind.can_render_environment(),
                prefix.kind.is_instance_universe(),
                prefix.kind.in_star_field(),
                json(&prefix.base, prefix.remainder)
            ),
            None => println!("{{\"kind\":null,\"payload_hex\":\"{}\"}}", hex(&bytes)),
        }
        return Ok(true);
    }
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
    println!("{}", json(&prefix, &bytes[prefix.consumed_bytes..]));
    Ok(true)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|v| format!("{v:02x}")).collect()
}

fn json(prefix: &NativeBasePrefix, remainder: &[u8]) -> String {
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
    let tail = hex(remainder);
    format!(
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
    )
}
