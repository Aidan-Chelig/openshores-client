use super::*;

fn fields() -> RegistrationFields {
    RegistrationFields {
        atom_type: 0x16,
        solar_id: 9,
        timer_words: [123, -456],
        released: false,
    }
}

#[test]
fn release_invalidates_timer_without_deleting_identity_or_interval() {
    let mut tocker = TockerFields {
        common: fields(),
        raw_interval: u32::MAX,
    };
    tocker.release();
    assert_eq!(
        tocker,
        TockerFields {
            common: RegistrationFields {
                timer_words: [i64::MIN; 2],
                released: true,
                ..fields()
            },
            raw_interval: u32::MAX,
        }
    );
    let before = tocker.clone();
    tocker.release();
    assert_eq!(tocker, before);
}

#[test]
fn reactivation_does_not_restart_an_invalidated_timer_or_replace_type() {
    let mut ticker = fields();
    ticker.release();
    ticker.update_existing(0);
    assert_eq!(
        ticker,
        RegistrationFields {
            solar_id: 0,
            timer_words: [i64::MIN; 2],
            released: false,
            ..fields()
        }
    );
}

#[test]
fn existing_tocker_update_preserves_timer_and_all_unsigned_interval_bits() {
    for interval in [0, 1, 0x80000000, u32::MAX] {
        let mut tocker = TockerFields {
            common: fields(),
            raw_interval: 3,
        };
        tocker.update_existing(42, interval);
        assert_eq!(
            tocker,
            TockerFields {
                common: RegistrationFields {
                    solar_id: 42,
                    ..fields()
                },
                raw_interval: interval
            }
        );
    }
}
