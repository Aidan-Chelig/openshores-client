//! Established native item-state observations, not a complete world snapshot.
use openshores_client::world::native_capture::{CaptureLimits, NativeCapture};
use std::{fs::File, io::BufReader};

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    if args.first().map(String::as_str) != Some("replay-native-items") {
        return Ok(false);
    }
    if args.len() != 2 {
        return Err("usage: replay-native-items CAPTURE_FILE".into());
    }
    let capture = NativeCapture::read(
        &mut BufReader::new(File::open(&args[1])?),
        CaptureLimits::default(),
    )?;
    let replay = capture.replay_items()?;
    // Whole capture and replay are validated before the first output line.
    println!(
        "{{\"item_entities\":{},\"decoded_updates\":{},\"opaque_updates\":{}}}",
        replay.items.len(),
        replay.decoded_updates,
        replay.opaque_updates
    );
    for (id, state) in replay.items {
        let item = state
            .item
            .as_ref()
            .map(|item| {
                format!(
                    "{{\"type_id\":{},\"commodity\":{},\"name_present\":{}}}",
                    item.extension.type_id(),
                    item.common.commodity,
                    item.common.name.is_some()
                )
            })
            .unwrap_or("null".into());
        let owner = state
            .ownership_id
            .map(|id| id.to_string())
            .unwrap_or("null".into());
        let field_39c = state
            .field_39c
            .map(|value| value.to_string())
            .unwrap_or("null".into());
        println!(
            "{{\"id\":{},\"item\":{},\"ownership_id\":{},\"field_3e0_known\":{},\"field_3e8_known\":{},\"field_39c\":{},\"opaque_updates\":{}}}",
            id,
            item,
            owner,
            state.field_3e0.is_some(),
            state.field_3e8.is_some(),
            field_39c,
            state.opaque_updates
        );
    }
    Ok(true)
}
