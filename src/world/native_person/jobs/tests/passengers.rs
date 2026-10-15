use super::*;

#[test]
fn passenger_families_keep_the_distinct_extra_int_string_and_id_tails() {
    let kinds = [16, 17, 18, 21, 22, 23];
    let mut list = vec![0, 6];
    for kind in kinds {
        let mut bytes = vec![];
        bytes.extend(2_440_588_u32.to_be_bytes());
        bytes.extend(123_u32.to_be_bytes());
        bytes.push(2);
        bytes.extend(u32::MAX.to_be_bytes());
        string(None, &mut bytes);
        bytes.extend(i32::MIN.to_be_bytes());
        if kind <= 18 {
            bytes.extend(i32::MAX.to_be_bytes());
        }
        string(Some(""), &mut bytes);
        bytes.extend(77_u32.to_be_bytes());
        string(Some("Passengers 🌊"), &mut bytes);
        if kind >= 21 {
            string(Some("extra"), &mut bytes);
        }
        if kind == 22 {
            bytes.extend(88_u32.to_be_bytes());
        }
        let length = bytes.len();
        bytes.push(0xee);
        let mut input = QtReader::new(&bytes);
        let job = NativeJob::read_qt7(kind, &mut input).unwrap();
        let delivery = NativeJobPassengerDelivery {
            id_10: u32::MAX,
            string_18: None,
            field_20: i32::MIN,
            field_24: i32::MAX,
            string_28: Some(String::new()),
            id_30: 77,
            string_38: Some("Passengers 🌊".into()),
        };
        let pickup = NativeJobPassengerPickup {
            id_10: u32::MAX,
            string_18: None,
            field_20: i32::MIN,
            string_28: Some(String::new()),
            id_30: 77,
            string_38: Some("Passengers 🌊".into()),
            string_40: Some("extra".into()),
        };
        assert_eq!(job.time_08, 123);
        assert_eq!(
            job.body,
            if kind <= 18 {
                NativeJobBody::DeliverPassengers(delivery)
            } else if kind == 22 {
                NativeJobBody::PickUpPassengersOverpopulated { pickup, id_48: 88 }
            } else {
                NativeJobBody::PickUpPassengers(pickup)
            }
        );
        assert_eq!(input.read_u8().unwrap(), 0xee);
        for end in 0..length {
            let mut input = QtReader::new(&bytes[..end]);
            assert_eq!(
                NativeJob::read_qt7(kind, &mut input).unwrap_err().kind(),
                io::ErrorKind::UnexpectedEof
            );
            assert_eq!(input.remaining(), end);
        }
        list.push(kind);
        list.extend(&bytes[..length]);
    }
    list.push(0xee);
    let mut jobs = NativeJobList::default();
    let mut input = QtReader::new(&list);
    assert_eq!(jobs.receive_qt7(&mut input).unwrap().stop, None);
    assert_eq!(jobs.0.iter().map(|job| job.kind).collect::<Vec<_>>(), kinds);
    assert_eq!(input.read_u8().unwrap(), 0xee);
}
