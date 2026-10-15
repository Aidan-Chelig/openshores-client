use super::*;

pub(super) const PROFILES: [FloatEncoding; 3] = [
    FloatEncoding::TypeWidths,
    FloatEncoding::SinglePrecision,
    FloatEncoding::DoublePrecision,
];

pub(super) fn float(out: &mut Vec<u8>, value: f64, profile: FloatEncoding) {
    if profile == FloatEncoding::DoublePrecision {
        out.extend(value.to_be_bytes());
    } else {
        out.extend((value as f32).to_be_bytes());
    }
}

pub(super) fn vector(out: &mut Vec<u8>, values: [f64; 3], profile: FloatEncoding) {
    for value in values {
        float(out, value, profile);
    }
}

pub(super) fn full(out: &mut Vec<u8>, kind: u8, profile: FloatEncoding) {
    out.push(kind);
    if !(1..=43).contains(&kind) {
        return;
    }
    out.extend(0xff012345_u32.to_be_bytes());
    vector(out, [3.5, -0., 1.0000000001], profile);
    match kind {
        31 => (),
        2 => {
            out.extend((-93_i32).to_be_bytes());
            out.push(0xfe);
            out.extend([0, 0, 0, 3, 0, 0xff, 0x80, 0x81]);
        }
        32 => {
            out.extend((-94_i32).to_be_bytes());
            out.push(0xfd);
            out.push(0); // no QString
            out.extend(111_i32.to_be_bytes());
            out.push(0xfc);
            out.extend((-222_i32).to_be_bytes());
            float(out, 4.75, profile);
            out.push(0);
            out.extend(333_i32.to_be_bytes());
        }
        18 => {
            out.extend((-95_i32).to_be_bytes());
            out.push(0xfb);
            out.extend(444_i32.to_be_bytes());
            out.extend(u32::MAX.to_be_bytes());
            out.extend([0, 0x80, 1]);
            out.extend((-555_i32).to_be_bytes());
            out.extend(0_u32.to_be_bytes());
            float(out, 5.25, profile);
            out.push(0);
            for value in [1.0000000001_f64, -0., 6.75] {
                if profile == FloatEncoding::SinglePrecision {
                    out.extend((value as f32).to_be_bytes());
                } else {
                    out.extend(value.to_be_bytes());
                }
            }
            out.extend(666_i32.to_be_bytes());
            float(out, 7.5, profile);
        }
        _ => panic!("no fixture for this known type"),
    }
}

pub(super) fn list(out: &mut Vec<u8>, profile: FloatEncoding) {
    out.extend(4_u16.to_be_bytes());
    for (kind, suffix) in [
        (31, &[][..]),
        (3, &[0xfe][..]),
        (2, &[0xfd, 3][..]),
        (32, &[0xfc, 0xfb][..]),
    ] {
        out.push(kind);
        out.extend(u32::from(kind).to_be_bytes());
        vector(out, [1., 2., 3.], profile);
        out.extend(suffix);
    }
}

pub(super) fn sensor(flags: u16, kind: u8, profile: FloatEncoding) -> Vec<u8> {
    let mut out = flags.to_be_bytes().to_vec();
    full(&mut out, kind, profile);
    vector(&mut out, [1.0000000001, -0., -10.5], profile);
    float(&mut out, 11.25, profile);
    float(&mut out, -12.75, profile);
    out.extend(0xfedcba98_u32.to_be_bytes());
    list(&mut out, profile);
    out
}

pub(super) fn weapon(flags: u16, kind: u8, profile: FloatEncoding) -> Vec<u8> {
    let mut out = flags.to_be_bytes().to_vec();
    out.push(0xfe);
    full(&mut out, kind, profile);
    vector(&mut out, [1.0000000001, -0., -10.5], profile);
    float(&mut out, 11.25, profile);
    out.push(0xfd);
    float(&mut out, -12.75, profile);
    out.extend(0x80ff_u16.to_be_bytes());
    out.extend(0xffff_u16.to_be_bytes());
    out.extend(i32::MIN.to_be_bytes());
    out.extend(0xfedcba98_u32.to_be_bytes());
    list(&mut out, profile);
    out
}

pub(super) fn transporter(first: u8, flags: u16, kind: u8, profile: FloatEncoding) -> Vec<u8> {
    let mut out = vec![first, 0xfe];
    if first & 0x10 != 0 {
        full(&mut out, kind, profile);
    }
    if first & 0x20 != 0 {
        vector(&mut out, [21.5, -22.75, 1.0000000001], profile);
    }
    vector(&mut out, [1.0000000001, -0., -10.5], profile);
    float(&mut out, 11.25, profile);
    out.extend(flags.to_be_bytes());
    float(&mut out, -12.75, profile);
    out.extend(0x80ff_u16.to_be_bytes());
    out.extend(0xfedcba98_u32.to_be_bytes());
    list(&mut out, profile);
    out
}

pub(super) fn bit(flags: u16, position: u16) -> u8 {
    u8::from(flags & (1 << position) != 0)
}

pub(super) fn assert_list(list: &NativeSensorBlipMinList) {
    use crate::world::native_sensor_blip::NativeSensorBlipMinBody;
    assert_eq!(list.0.len(), 4);
    for (entry, kind) in list.0.iter().zip([31, 3, 2, 32]) {
        assert_eq!(entry.kind, kind);
        assert_eq!(entry.id_08, u32::from(kind));
        assert_eq!(entry.position_10, [1., 2., 3.]);
    }
    assert_eq!(list.0[0].body, NativeSensorBlipMinBody::Base);
    assert_eq!(
        list.0[1].body,
        NativeSensorBlipMinBody::Empire { field_2c: 0xfe }
    );
    assert_eq!(
        list.0[2].body,
        NativeSensorBlipMinBody::Animal {
            field_2c: 0xfd,
            field_40: 1,
            field_28: 0
        }
    );
    assert_eq!(
        list.0[3].body,
        NativeSensorBlipMinBody::Bd {
            field_2c: 0xfc,
            field_34: 0xfb
        }
    );
}

pub(super) fn assert_selection(
    selection: &NativeStationBlipSelection,
    kind: u8,
    profile: FloatEncoding,
) {
    use crate::world::native_sensor_blip::{NativeBlipCreatureSuffix, NativeBlipFullBody};
    assert_eq!(selection.kind, Some(kind));
    if !(1..=43).contains(&kind) {
        assert!(selection.value.is_none());
        return;
    }
    let full = selection.value.as_ref().unwrap();
    assert_eq!(full.kind, kind);
    assert_eq!(full.id_08, 0xff012345);
    assert_eq!(full.position_10, [3.5, -0., 1.]);
    match &full.body {
        NativeBlipFullBody::Anomaly => assert_eq!(kind, 31),
        NativeBlipFullBody::Creature(value) => {
            assert_eq!(kind, 2);
            assert_eq!(value.empire.field_28, -93);
            assert_eq!(value.empire.field_2c, 0xfe);
            assert_eq!(value.species_30.0, Some(vec![0, 0xff, 0x80]));
            assert_eq!(
                value.suffix,
                NativeBlipCreatureSuffix::Animal { field_40: true }
            );
        }
        NativeBlipFullBody::Bd(value) => {
            assert_eq!(kind, 32);
            assert_eq!(value.empire.field_28, -94);
            assert_eq!(value.empire.field_2c, 0xfd);
            assert_eq!(
                (
                    value.field_35,
                    value.field_30,
                    value.field_34,
                    value.field_38
                ),
                (0, 111, 0xfc, -222)
            );
            assert_eq!(value.field_40, None);
            assert_eq!(
                (value.field_48, value.field_4c, value.field_50),
                (4.75, 1, 333)
            );
        }
        NativeBlipFullBody::Spacecraft { fields, shield_74 } => {
            assert_eq!(kind, 18);
            assert_eq!(
                (
                    fields.empire.field_28,
                    fields.empire.field_2c,
                    fields.field_30
                ),
                (-95, 0xfb, 444)
            );
            assert_eq!(fields.field_38, None);
            assert!(!fields.field_40);
            assert!(fields.field_41);
            assert!(fields.field_42);
            assert_eq!(fields.field_44, -555);
            assert_eq!(fields.field_48, Some(String::new()));
            assert_eq!(
                (fields.field_50, fields.field_54, fields.field_70),
                (5.25, 1, 666)
            );
            assert_eq!(
                fields.point_58[0],
                if profile == FloatEncoding::SinglePrecision {
                    1.
                } else {
                    1.0000000001
                }
            );
            assert_eq!(fields.point_58[1].to_bits(), (-0_f64).to_bits());
            assert_eq!(fields.point_58[2], 6.75);
            assert_eq!(*shield_74, Some(7.5));
        }
        _ => panic!("unexpected full body"),
    }
}
