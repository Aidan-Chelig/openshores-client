use super::*;
use crate::{
    protocol::QtWriter,
    world::{
        native_item::{NativeItemBody, NativeItemExtension, NativeItemValue},
        native_person::{NativeGear, NativeGearSlot, NativePersonSuffixState},
    },
};

fn fixture(profile: FloatEncoding, name: Option<&str>) -> (Vec<u8>, [u64; 3]) {
    let mut bytes = vec![10, 0, 0, 3, 1];
    bytes.extend(u32::MAX.to_be_bytes());
    let expected = if profile == FloatEncoding::SinglePrecision {
        let words = [0x7fc0_1234_u32, 0x8000_0000, 0x3f80_0001];
        for word in words {
            bytes.extend(word.to_be_bytes());
        }
        words.map(|word| f64::from(f32::from_bits(word)).to_bits())
    } else {
        let words = [
            0x7ff8_0000_0000_1234_u64,
            0x8000_0000_0000_0000,
            0x3ff0_0000_0000_0001,
        ];
        for word in words {
            bytes.extend(word.to_be_bytes());
        }
        words
    };
    let mut writer = QtWriter::default();
    writer.write_string(name).unwrap();
    bytes.extend(writer.into_bytes());
    (bytes, expected)
}

#[test]
fn real_point_overloads_keep_bits_profiles_nullable_strings_and_atomic_truncation() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        for name in [None, Some(""), Some("Galaxy 🌊")] {
            let (mut bytes, expected) = fixture(profile, name);
            let length = bytes.len();
            bytes.push(0xee);
            let mut input = QtReader::new(&bytes);
            let item = NativeItemValue::read_with_profile(&mut input, profile)
                .unwrap()
                .unwrap();
            assert_eq!(item.consumed_bytes, length);
            assert_eq!(item.extension.type_id(), 10);
            let NativeItemExtension::GalaxyLocation(location) = item.extension else {
                panic!("wrong extension");
            };
            assert_eq!(location.id_4c, u32::MAX);
            assert_eq!(location.point_50_bits, expected);
            assert_eq!(location.point_50().map(f64::to_bits), expected);
            assert_eq!(location.string_68.as_deref(), name);
            assert_eq!(input.read_u8().unwrap(), 0xee);
            for end in 0..length {
                let mut input = QtReader::new(&bytes[..end]);
                assert_eq!(
                    NativeItemValue::read_with_profile(&mut input, profile)
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::UnexpectedEof
                );
                assert_eq!(input.remaining(), end);
            }
        }
    }
}

#[test]
fn gear_body_and_person_inventory_propagate_actual_float_profile() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let (item, expected) = fixture(profile, Some("Galaxy 🌊"));
        let mut bytes = vec![0xab];
        bytes.extend(&item);
        bytes.push(0xee);
        let mut input = QtReader::new(&bytes);
        let slot = NativeGearSlot::read_with_profile(&mut input, profile)
            .unwrap()
            .unwrap();
        assert_eq!(slot.field_08, 11);
        assert_eq!(slot.field_09, 10);
        let NativeItemExtension::GalaxyLocation(location) = slot.item.extension else {
            panic!("wrong item");
        };
        assert_eq!(location.point_50_bits, expected);
        assert_eq!(input.read_u8().unwrap(), 0xee);
        let mut bytes = vec![1, 0xab];
        bytes.extend(&item);
        bytes.push(0xee);
        let mut input = QtReader::new(&bytes);
        let mut gear = NativeGear::default();
        let report = gear.receive_with_profile(&mut input, profile).unwrap();
        assert!(report.null_slots.is_empty());
        assert_eq!(gear.0.len(), 1);
        assert_eq!(input.read_u8().unwrap(), 0xee);
        // This body must decode fully before the still-unsupported real weight stage.
        let mut bytes = vec![0xc0, 1, 1, 0xab];
        bytes.extend(&item);
        bytes.extend([0, 0xee]);
        let mut input = QtReader::new(&bytes);
        let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
        assert_eq!(
            state
                .receive_suffix(&mut input, profile, std::time::Instant::now)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(state.gear_1390.unwrap().0.len(), 1);
        assert_eq!(state.weight_13a8, None);
        assert_eq!(state.domain_1018, None);
        assert_eq!(input.remaining_bytes(), [0, 0xee]);
        let mut bytes = vec![0x88, 0x40, 0xc5];
        bytes.extend(1_u32.to_be_bytes());
        bytes.extend(&item);
        bytes.extend([0xab, 0xee]);
        let mut input = QtReader::new(&bytes);
        let mut state = NativePersonSuffixState::with_prior_fields(0, 0);
        let report = state
            .receive_suffix(&mut input, profile, std::time::Instant::now)
            .unwrap();
        assert!(report.inventory.unwrap().obsolete_items.is_empty());
        let NativeItemExtension::GalaxyLocation(location) =
            &state.inventory_1458.unwrap().0[&3][0].extension
        else {
            panic!("wrong inventory item");
        };
        assert_eq!(location.point_50_bits, expected);
        assert_eq!(state.field_83d, Some(11));
        assert_eq!(input.read_u8().unwrap(), 0xee);
        // DaItem body wrapper must likewise stop before ownership, not swallow it.
        let mut bytes = vec![0x81];
        bytes.extend(&item);
        bytes.extend(88_u32.to_be_bytes());
        bytes.extend(u32::MAX.to_be_bytes());
        bytes.extend(u32::MAX.to_be_bytes());
        bytes.push(0xee);
        let mut input = QtReader::new(&bytes);
        assert!(
            NativeItemBody::read_with_profile(&mut input, profile)
                .unwrap()
                .unwrap()
                .item
                .is_some()
        );
        assert_eq!(input.read_u8().unwrap(), 0xee);
    }
}
