//! Explicit item-value diagnostics, never a connected command.
use super::positions;
use openshores_client::{
    protocol::QtReader,
    world::native_item::{NativeItemCommon, NativeItemExtension, NativeItemValue, NativeWeapon},
};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(super) fn run(args: &[String]) -> Result<bool, Box<dyn std::error::Error>> {
    let command = args.first().map(String::as_str);
    if !matches!(
        command,
        Some("inspect-native-item-common" | "inspect-native-item-value")
    ) {
        return Ok(false);
    }
    if args.len() != 2 {
        return Err(
            "usage: inspect-native-item-common|inspect-native-item-value HEX_RECORD".into(),
        );
    }
    let input_args = vec![args[0].clone(), "type-widths".into(), args[1].clone()];
    let (_, bytes) = positions::input(&input_args, 4096)?;
    if command == Some("inspect-native-item-value") {
        match NativeItemValue::read(&mut QtReader::new(&bytes))? {
            None => println!("{{\"type_id\":null,\"payload_hex\":\"{}\"}}", hex(&bytes)),
            Some(item) => {
                let extension = match &item.extension {
                    NativeItemExtension::Plain | NativeItemExtension::Ammo => "{}".into(),
                    NativeItemExtension::Weapon(weapon) => weapon_json(weapon),
                    NativeItemExtension::WeaponAmmo { weapon, fields_80 } => format!(
                        "{{\"weapon\":{},\"fields_80_hex\":\"{}\"}}",
                        weapon_json(weapon),
                        hex(fields_80)
                    ),
                    NativeItemExtension::WeaponState { weapon, field_80 } => format!(
                        "{{\"weapon\":{},\"field_80\":{}}}",
                        weapon_json(weapon),
                        field_80
                    ),
                    NativeItemExtension::State { field_4c } => {
                        format!("{{\"field_4c\":{field_4c}}}")
                    }
                    NativeItemExtension::GalaxyLocation(location) => format!(
                        "{{\"id_4c\":{},\"point_50_bits_hex\":[\"{:016x}\",\"{:016x}\",\"{:016x}\"],\"string_68_utf8_hex\":{}}}",
                        location.id_4c,
                        location.point_50_bits[0],
                        location.point_50_bits[1],
                        location.point_50_bits[2],
                        string_hex(location.string_68.as_deref())
                    ),
                    NativeItemExtension::StateMessage { field_4c, field_50 } => format!(
                        "{{\"field_4c\":{},\"field_50_utf8_hex\":{}}}",
                        field_4c,
                        string_hex(field_50.as_deref())
                    ),
                };
                println!(
                    "{{\"type_id\":{},\"common\":{},\"extension\":{},\"consumed_bytes\":{},\"tail_hex\":\"{}\"}}",
                    item.extension.type_id(),
                    common_json(&item.common, &[]),
                    extension,
                    item.consumed_bytes,
                    hex(&bytes[item.consumed_bytes..])
                );
            }
        }
    } else {
        let item = NativeItemCommon::read(&mut QtReader::new(&bytes))?;
        println!("{}", common_json(&item, &bytes[item.consumed_bytes..]));
    }
    Ok(true)
}

fn string_hex(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", hex(value.as_bytes())))
        .unwrap_or("null".into())
}

fn weapon_json(weapon: &NativeWeapon) -> String {
    let fields = weapon.fresh_fields();
    // Raw wire groups remain separate from reconstructed overlapping destinations.
    let groups = weapon
        .groups
        .iter()
        .map(|group| {
            group
                .as_ref()
                .map(|group| {
                    format!(
                        "[{}, {}, {}, {}, {}, {}]",
                        group.byte_0,
                        group.short_1,
                        group.variable_2,
                        group.short_3,
                        group.byte_4,
                        group.short_5
                    )
                })
                .unwrap_or("null".into())
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"flags\":{},\"field_4c\":{},\"wire_field_66\":{},\"groups\":[{}],\"fresh_field_50\":{},\"fresh_field_5c\":{},\"fresh_field_6c\":{},\"fresh_field_78\":{},\"consumed_bytes\":{}}}",
        weapon.flags,
        weapon.field_4c,
        weapon
            .wire_field_66
            .map(|field| field.to_string())
            .unwrap_or("null".into()),
        groups,
        fields.field_50,
        fields.field_5c,
        fields.field_6c,
        fields.field_78,
        weapon.consumed_bytes
    )
}

fn common_json(item: &NativeItemCommon, tail: &[u8]) -> String {
    // UTF-8 hex avoids escaping ambiguity; null, empty, and omission remain distinct.
    let name = item
        .name
        .as_ref()
        .and_then(|name| name.as_ref())
        .map(|name| format!("\"{}\"", hex(name.as_bytes())))
        .unwrap_or("null".into());
    let field_14 = item
        .field_14
        .map(|value| value.to_string())
        .unwrap_or("null".into());
    format!(
        "{{\"flags\":{},\"commodity\":{},\"field_0c\":{},\"field_10\":{},\"field_14\":{},\"name_present\":{},\"name_utf8_hex\":{},\"wire_field_20\":{},\"field_20\":{},\"field_24\":{},\"field_28\":{},\"consumed_bytes\":{},\"tail_hex\":\"{}\"}}",
        item.flags,
        item.commodity,
        item.field_0c,
        item.field_10,
        field_14,
        item.name.is_some(),
        name,
        item.wire_field_20,
        item.field_20(),
        item.field_24,
        item.field_28,
        item.consumed_bytes,
        hex(tail)
    )
}
