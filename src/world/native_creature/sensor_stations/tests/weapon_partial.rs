use super::*;

#[test]
fn weapon_errors_retain_prior_writes_and_do_not_clear_selection_before_type() {
    for profile in PROFILES {
        let mut old = NativeStationWeapon::default();
        let _result = old
            .receive_wire(&mut QtReader::new(&weapon(0, 18, profile)), profile)
            .unwrap();
        old.field_41 = Some(90);
        old.vector_20 = Some([91., 92., 93.]);
        old.field_38 = Some(94.);
        old.field_3c = Some(95);
        old.field_4c = Some(96.);
        old.field_50 = Some(97);
        old.field_a2 = Some(98);
        old.field_44 = Some(99);
        old.id_1c = Some(100);
        let bytes = weapon(0xffff, 2, profile);
        let mut full_body = Vec::new();
        full(&mut full_body, 2, profile);
        let full_end = 3 + full_body.len();
        let width = if profile == FloatEncoding::DoublePrecision {
            8
        } else {
            4
        };
        let vector_end = full_end + 3 * width;
        let float38_end = vector_end + width;
        let byte3c_end = float38_end + 1;
        let float4c_end = byte3c_end + width;
        let short50_end = float4c_end + 2;
        let shorta2_end = short50_end + 2;
        let int44_end = shorta2_end + 4;
        let id_end = int44_end + 4;
        let first_min_end = id_end + 2 + 1 + 4 + 3 * width;
        for end in 0..bytes.len() {
            let mut value = old.clone();
            assert!(
                value
                    .receive_wire(&mut QtReader::new(&bytes[..end]), profile)
                    .is_err()
            );
            assert_eq!(
                value.flags,
                if end >= 2 {
                    Some(NativeStationWeaponFlags {
                        fields_14: [1; 4],
                        field_18: 1,
                        field_48: 1,
                        fields_3d: [1; 4],
                    })
                } else {
                    old.flags
                }
            );
            assert_eq!(
                value.field_41,
                if end >= 3 { Some(0xfe) } else { old.field_41 }
            );
            if end < 4 {
                assert_eq!(value.blip_70, old.blip_70);
            } else if end < full_end {
                assert_eq!(value.blip_70.kind, Some(2));
                assert!(value.blip_70.value.is_none());
            } else {
                assert_selection(&value.blip_70, 2, profile);
            }
            assert_eq!(
                value.vector_20,
                if end >= vector_end {
                    Some([1., -0., -10.5])
                } else {
                    old.vector_20
                }
            );
            assert_eq!(
                value.field_38,
                if end >= float38_end {
                    Some(11.25)
                } else {
                    old.field_38
                }
            );
            assert_eq!(
                value.field_3c,
                if end >= byte3c_end {
                    Some(0xfd)
                } else {
                    old.field_3c
                }
            );
            assert_eq!(
                value.field_4c,
                if end >= float4c_end {
                    Some(-12.75)
                } else {
                    old.field_4c
                }
            );
            assert_eq!(
                value.field_50,
                if end >= short50_end {
                    Some(0x80ff)
                } else {
                    old.field_50
                }
            );
            assert_eq!(
                value.field_a2,
                if end >= shorta2_end {
                    Some(0xffff)
                } else {
                    old.field_a2
                }
            );
            assert_eq!(
                value.field_44,
                if end >= int44_end {
                    Some(i32::MIN)
                } else {
                    old.field_44
                }
            );
            assert_eq!(
                value.id_1c,
                if end >= id_end {
                    Some(0xfedcba98)
                } else {
                    old.id_1c
                }
            );
            if end < id_end {
                assert_eq!(value.list_78, old.list_78);
            } else if end < first_min_end {
                assert!(value.list_78.0.is_empty());
            } else {
                assert_eq!(value.list_78.0[0].kind, 31);
                assert!(value.list_78.0.len() < 4);
            }
        }
    }
}

#[test]
fn raw_station_bytes_widen_without_sign_extension_or_boolean_clamping() {
    for profile in PROFILES {
        for byte in 0..=255 {
            let mut bytes = transporter(0, 0, 0, profile);
            bytes[1] = byte;
            let mut value = NativeStationTransporter::default();
            let _result = value
                .receive_wire(&mut QtReader::new(&bytes), profile)
                .unwrap();
            assert_eq!(value.field_1c, Some(u32::from(byte)));
            let mut bytes = weapon(0, 0, profile);
            bytes[2] = byte;
            let mut value = NativeStationWeapon::default();
            let _result = value
                .receive_wire(&mut QtReader::new(&bytes), profile)
                .unwrap();
            assert_eq!(value.field_41, Some(byte));
        }
    }
}
