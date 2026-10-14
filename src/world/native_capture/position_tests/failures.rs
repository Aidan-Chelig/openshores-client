use super::*;
use crate::world::native_tick_tock::{RegistrationAction, SolarSystem};

struct RegisteringOwner;
struct RegisteringContext;
impl ReparentRegistrationContext for RegisteringOwner {
    type Context<'a> = RegisteringContext;
    fn context<'a>(
        &'a mut self,
        graph: &'a mut NativeNormalSubatoms,
        id: u32,
    ) -> RegisteringContext {
        assert_eq!(graph.node(id).unwrap().position.position.parent, 7);
        RegisteringContext
    }
}
impl StoreContext for RegisteringContext {
    fn must_not_tick_tock(&mut self, _: &RegistrationStore) -> bool {
        false
    }
    fn is_ticker(&mut self, _: &RegistrationStore) -> bool {
        true
    }
    fn is_tocker(&mut self, _: &RegistrationStore) -> u32 {
        123
    }
    fn find_parent_solar_system(&mut self, _: &RegistrationStore) -> Option<SolarSystem> {
        Some(SolarSystem {
            id: 99,
            ticking_contents: true,
            field_3a9: 0,
        })
    }
    fn is_ticking_contents(&mut self, _: &RegistrationStore, solar: SolarSystem) -> bool {
        solar.ticking_contents
    }
    fn solar_field_3a9(&mut self, _: &RegistrationStore, solar: SolarSystem) -> u8 {
        solar.field_3a9
    }
    fn atom_type(&mut self, _: &RegistrationStore) -> u8 {
        0x16
    }
}

#[test]
fn replay_failure_reports_prior_outcomes_but_retains_failed_frames_partial_effects() {
    let capture = capture(&[
        packet(1, 0, 12.5, 1, true),
        packet(1, 7, 99., 2, true),
        packet(1, 7, 3., 3, true),
    ]);
    let mut graph = NativeNormalSubatoms::new(
        [(
            1,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: 0,
                position: NativePositionState::default(),
            },
        )],
        [
            (
                0,
                NativeChildList {
                    available: true,
                    children: vec![1],
                },
            ),
            (
                7,
                NativeChildList {
                    available: true,
                    children: vec![],
                },
            ),
        ],
    )
    .unwrap();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::with_limit(1).unwrap();
    store
        .apply(RegistrationAction::UpdateOrAddTocker {
            id: 3,
            atom_type: 2,
            solar_id: 4,
            raw_interval: 5,
        })
        .unwrap();
    let mut owner = RegisteringOwner;
    let now = Instant::now();
    let mut evaluated = Vec::new();
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    let error = capture
        .replay_positions(&mut receiver, |index, _| {
            evaluated.push(index);
            (
                DestinationContext::ExistingParentLists,
                now + std::time::Duration::from_millis(index as u64),
            )
        })
        .unwrap_err();
    assert_eq!(error.frame, Some(1));
    assert_eq!(evaluated, [0, 1]);
    assert_eq!(error.completed.positions.len(), 1);
    assert_eq!(error.completed.positions[0].frame, 0);
    assert_eq!(graph.node(1).unwrap().position.position.parent, 7);
    assert_eq!(graph.node(1).unwrap().position.position.local[3][0], 99.);
    assert!(!graph.node(1).unwrap().position.environment_cache_valid);
    assert_eq!(
        graph
            .node(1)
            .unwrap()
            .position
            .position_update_age_at(now + std::time::Duration::from_millis(1)),
        Some(std::time::Duration::ZERO)
    );
    assert_eq!(store.ticker(1).unwrap().solar_id, 99);
    assert!(store.tocker(1).is_none());
    assert_eq!(store.tocker(3).unwrap().raw_interval, 5);
    assert!(queue.is_empty());
}
