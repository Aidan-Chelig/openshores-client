use super::*;
use crate::world::native_tick_tock::RegistrationAction;

#[derive(Default)]
struct WorldProbe {
    blocked: bool,
    calls: Vec<&'static str>,
}
impl StoreContext for WorldProbe {
    fn must_not_tick_tock(&mut self, _: &RegistrationStore) -> bool {
        self.calls.push("blocked");
        self.blocked
    }
    fn is_ticker(&mut self, _: &RegistrationStore) -> bool {
        self.calls.push("ticker");
        true
    }
    fn is_tocker(&mut self, store: &RegistrationStore) -> u32 {
        self.calls.push("tocker");
        assert!(store.ticker(7).is_some_and(|fields| !fields.released));
        u32::MAX
    }
    fn find_parent_solar_system(&mut self, _: &RegistrationStore) -> Option<SolarSystem> {
        self.calls.push("parent");
        Some(SolarSystem {
            id: 99,
            ticking_contents: true,
            field_3a9: 0,
        })
    }
    fn is_ticking_contents(&mut self, _: &RegistrationStore, solar: SolarSystem) -> bool {
        self.calls.push("contents");
        solar.ticking_contents
    }
    fn solar_field_3a9(&mut self, store: &RegistrationStore, solar: SolarSystem) -> u8 {
        self.calls.push("solar-byte");
        assert!(store.ticker(7).is_some());
        solar.field_3a9
    }
    fn atom_type(&mut self, store: &RegistrationStore) -> u8 {
        self.calls.push("type");
        if store.ticker(7).is_some() {
            0x17
        } else {
            0x16
        }
    }
}

#[test]
fn dispatcher_mutates_real_tables_before_later_world_getters() {
    let mut store = RegistrationStore::default();
    let mut world = WorldProbe::default();
    store.apply_tick_tock(7, &mut world).unwrap();
    assert_eq!(store.ticker(7).unwrap().atom_type, 0x16);
    assert_eq!(store.tocker(7).unwrap().common.atom_type, 0x17);
    assert_eq!(store.tocker(7).unwrap().raw_interval, u32::MAX);
    assert_eq!(
        world.calls,
        [
            "blocked",
            "ticker",
            "parent",
            "contents",
            "type",
            "tocker",
            "solar-byte",
            "type"
        ]
    );
    let mut world = WorldProbe {
        blocked: true,
        ..WorldProbe::default()
    };
    store.apply_tick_tock(7, &mut world).unwrap();
    assert_eq!(world.calls, ["blocked"]);
    assert!(store.ticker(7).unwrap().released);
    assert!(store.tocker(7).unwrap().common.released);
    assert_eq!(store.cleanup_pending(), (true, true));
}

#[test]
fn first_storage_failure_stops_before_tocker_getters() {
    let mut store = RegistrationStore::with_limit(1).unwrap();
    store
        .apply(RegistrationAction::UpdateOrAddTicker {
            id: 1,
            atom_type: 2,
            solar_id: 3,
        })
        .unwrap();
    let mut world = WorldProbe::default();
    assert!(store.apply_tick_tock(7, &mut world).is_err());
    assert_eq!(
        world.calls,
        ["blocked", "ticker", "parent", "contents", "type"]
    );
    assert!(store.ticker(7).is_none());
    assert!(store.tocker(7).is_none());
    assert_eq!(store.ticker(1).unwrap().solar_id, 3);
}

#[test]
fn later_storage_failure_retains_earlier_successful_registration() {
    let mut store = RegistrationStore::with_limit(1).unwrap();
    store
        .apply(RegistrationAction::UpdateOrAddTocker {
            id: 1,
            atom_type: 2,
            solar_id: 3,
            raw_interval: 4,
        })
        .unwrap();
    let mut world = WorldProbe::default();
    assert!(store.apply_tick_tock(7, &mut world).is_err());
    assert_eq!(store.ticker(7).unwrap().solar_id, 99);
    assert!(store.tocker(7).is_none());
    assert_eq!(store.tocker(1).unwrap().raw_interval, 4);
}
