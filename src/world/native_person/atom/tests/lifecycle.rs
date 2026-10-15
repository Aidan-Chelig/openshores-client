use super::*;
use crate::world::{
    native_position_updates::{DestinationContext, NativePositionReceiver},
    native_runaways::NativeRunaways,
    native_subatoms::{NativeAtomOwnerContext, NativeAtomOwnerStores, ReparentRegistrationContext},
    native_tick_tock::{
        SolarSystem,
        store::{RegistrationStore, StoreContext},
    },
};

#[derive(Default)]
struct Owner {
    assigned: Vec<u32>,
    registrations: usize,
}
struct Context;
impl ReparentRegistrationContext for Owner {
    type Context<'a> = Context;
    fn context<'a>(&'a mut self, _: &'a mut NativeNormalSubatoms, _: u32) -> Context {
        self.registrations += 1;
        Context
    }
}
impl NativeAtomOwnerContext for Owner {
    fn detach_owner(&mut self, _: u32) {
        panic!("fresh owner must not replace retained object")
    }
    fn assign_owner(&mut self, _: &NativeNormalSubatoms, handle: u32) {
        self.assigned.push(handle);
    }
}
// Explicit synthetic registration observations, not inferred native Person policy.
impl StoreContext for Context {
    fn must_not_tick_tock(&mut self, _: &RegistrationStore) -> bool {
        false
    }
    fn is_ticker(&mut self, _: &RegistrationStore) -> bool {
        true
    }
    fn is_tocker(&mut self, _: &RegistrationStore) -> u32 {
        0
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
        0x05
    }
}

#[test]
fn fresh_owner_add_then_retained_person_position_uses_wire_id_indexes() {
    let mut graph = NativeNormalSubatoms::new([], []).unwrap();
    let mut people = NativePersonObjects::default();
    people
        .construct_candidate(NativeAtomKind::Person, 90, &mut graph)
        .unwrap();
    people
        .get_mut(90)
        .unwrap()
        .base
        .apply_received_identity(NativeIdentityPrefix {
            id: 7,
            timestamp_ms: 123,
        });
    let mut lookup = NativeRecoveryLookup::new([], []).unwrap();
    let mut queue = NativeRunaways::default();
    let mut registrations = RegistrationStore::default();
    let mut retired = NativeRetiredAtoms::default();
    let mut owner = Owner::default();
    people
        .receive_candidate_base(
            90,
            &mut QtReader::new(&[0]),
            NativePersonCandidateReceiveContext {
                graph: &mut graph,
                lookup: &lookup,
                retired: &retired,
                position_epoch: Instant::now(),
                profile: FloatEncoding::TypeWidths,
            },
            |_| panic!("omitted position"),
            || panic!("omitted effects"),
        )
        .unwrap();
    people
        .add_candidate_owner(
            90,
            &mut graph,
            NativeAtomOwnerStores {
                lookup: &mut lookup,
                runaways: &mut queue,
                registrations: &mut registrations,
                retired: &mut retired,
            },
            &mut owner,
        )
        .unwrap();
    assert_eq!(owner.assigned, [90]);
    assert_eq!(lookup.find_atom(7), Some(90));
    assert!(people.get(90).unwrap().base.owner_attached);
    assert!(
        people
            .discard_candidate(90, &mut graph, &lookup, &retired)
            .is_err()
    );
    assert!(
        people
            .add_candidate_owner(
                90,
                &mut graph,
                NativeAtomOwnerStores {
                    lookup: &mut lookup,
                    runaways: &mut queue,
                    registrations: &mut registrations,
                    retired: &mut retired,
                },
                &mut owner
            )
            .is_err()
    );
    assert_eq!(owner.assigned, [90]);
    people
        .construct_candidate(NativeAtomKind::Person, 91, &mut graph)
        .unwrap();
    people
        .get_mut(91)
        .unwrap()
        .base
        .apply_received_identity(NativeIdentityPrefix {
            id: 7,
            timestamp_ms: 456,
        });
    assert!(
        people
            .add_candidate_owner(
                91,
                &mut graph,
                NativeAtomOwnerStores {
                    lookup: &mut lookup,
                    runaways: &mut queue,
                    registrations: &mut registrations,
                    retired: &mut retired,
                },
                &mut owner
            )
            .is_err()
    );
    assert!(!people.get(91).unwrap().base.owner_attached);
    people
        .discard_candidate(91, &mut graph, &lookup, &retired)
        .unwrap();
    assert_eq!(lookup.find_atom(7), Some(90));
    assert_eq!(people.len(), 1);
    assert_eq!(people.get(90).unwrap().base.received_time, 123);
    assert_eq!(owner.assigned, [90]);
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut registrations,
        owner: &mut owner,
    };
    let mut bytes = vec![1, 0x83];
    bytes.extend(12_u32.to_be_bytes());
    bytes.extend(12.5_f64.to_be_bytes());
    bytes.push(0xee);
    let mut input = QtReader::new(&bytes);
    people
        .receive_owned_base(
            90,
            &mut input,
            NativePersonOwnedReceiveContext {
                receiver: &mut receiver,
                lookup: &lookup,
                position_epoch: Instant::now(),
                profile: FloatEncoding::TypeWidths,
            },
            |_| DestinationContext::MissingDestinationLock,
            |_| Ok(()),
            || panic!("omitted effects"),
        )
        .unwrap();
    assert_eq!(input.read_u8().unwrap(), 0xee);
    assert!(receiver.runaways.contains(12, 7));
    assert!(!receiver.runaways.contains(12, 90));
    assert!(receiver.registrations.ticker(7).is_some());
    assert!(receiver.registrations.ticker(90).is_none());
    assert_eq!(
        people.get(90).unwrap().base.position,
        receiver.graph.node(90).unwrap().position
    );
    assert_eq!(people.get(90).unwrap().body.person.field_83d, Some(15));
}
