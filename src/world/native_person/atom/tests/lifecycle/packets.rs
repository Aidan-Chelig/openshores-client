//! Full unframed native packet fixtures; runtime observations remain synthetic.
use super::*;
use crate::world::{
    native_dispatch::{
        NativeAtomDispatchOutcome,
        person::{NativePersonDispatch, NativePersonDispatchHooks},
        receive_atom,
    },
    native_signals::{NativeSignalConnection, NativeSignalConnector},
};
mod failures;
mod runtime;
use runtime::Hooks;

#[derive(Default)]
struct Connector(Vec<(u32, NativeSignalConnection)>);
impl NativeSignalConnector for Connector {
    fn connect(&mut self, handle: u32, binding: NativeSignalConnection) -> io::Result<bool> {
        self.0.push((handle, binding));
        Ok(false)
    }
}
struct Harness {
    people: NativePersonObjects,
    graph: NativeNormalSubatoms,
    lookup: NativeRecoveryLookup,
    queue: NativeRunaways,
    registrations: RegistrationStore,
    retired: NativeRetiredAtoms,
    owner: Owner,
    connector: Connector,
    hooks: Hooks,
    profile: FloatEncoding,
}
impl Harness {
    fn new(profile: FloatEncoding) -> Self {
        Self {
            people: NativePersonObjects::default(),
            graph: NativeNormalSubatoms::new([], []).unwrap(),
            lookup: NativeRecoveryLookup::new([], []).unwrap(),
            queue: NativeRunaways::default(),
            registrations: RegistrationStore::default(),
            retired: NativeRetiredAtoms::default(),
            owner: Owner::default(),
            connector: Connector::default(),
            hooks: Hooks::default(),
            profile,
        }
    }
    fn receive(&mut self, payload: &[u8]) -> io::Result<Option<NativeAtomDispatchOutcome>> {
        receive_atom(
            payload,
            &mut NativePersonDispatch {
                people: &mut self.people,
                graph: &mut self.graph,
                lookup: &mut self.lookup,
                runaways: &mut self.queue,
                registrations: &mut self.registrations,
                retired: &mut self.retired,
                owner: &mut self.owner,
                connector: &mut self.connector,
                hooks: &mut self.hooks,
                raw_mode: 5,
                profile: self.profile,
            },
        )
    }
}
fn packet(time: i64, base_and_body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x05];
    bytes.extend(7_u32.to_be_bytes());
    bytes.extend(time.to_be_bytes());
    bytes.extend(base_and_body);
    bytes
}
fn complete(profile: FloatEncoding) -> Vec<u8> {
    let mut bytes = vec![0x76];
    bytes.extend((-11_i64).to_be_bytes());
    bytes.extend((-22_i64).to_be_bytes());
    bytes.extend([1, 0, 0, 7, 2, 2, 2]); // effects and controllers
    bytes.push(1);
    bytes.extend(u32::MAX.to_be_bytes());
    bytes.extend((-7_i32).to_be_bytes());
    bytes.push(0x80); // Unit
    bytes.extend(0x204_u16.to_be_bytes());
    bytes.extend(i16::MIN.to_be_bytes());
    bytes.extend([0xab, 0xff, 0x80]); // Creature
    bytes.extend([6, 0x29]);
    bytes.extend(77_u32.to_be_bytes());
    bytes.extend(u32::MAX.to_be_bytes());
    bytes.extend([0xfe, 0xff]);
    bytes.extend(1_u16.to_be_bytes());
    bytes.extend(0x8000_u16.to_be_bytes());
    bytes.extend(i32::MIN.to_be_bytes()); // Sentient skills
    bytes.extend([0x8c, 0x10]);
    bytes.extend(0x8001_u16.to_be_bytes());
    bytes.push(0xc0); // Person + inventory/nibbles
    if profile == FloatEncoding::SinglePrecision {
        bytes.extend(1.25_f32.to_be_bytes());
    } else {
        bytes.extend(1.25_f64.to_be_bytes());
    }
    bytes.extend(2_u32.to_be_bytes());
    bytes.extend([0, 21, 0xab]);
    packet(123, &bytes)
}

#[test]
fn complete_native_person_packets_construct_attach_receive_reuse_and_dispose_candidates() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let mut h = Harness::new(profile);
        let mut bytes = complete(profile);
        let length = bytes.len();
        bytes.push(0xee);
        let first = h.receive(&bytes).unwrap().unwrap();
        assert_eq!(
            first,
            NativeAtomDispatchOutcome {
                wire_id: 7,
                handle: 90,
                is_new: true,
                consumed_bytes: length
            }
        );
        let person = h.people.get(90).unwrap();
        assert!(person.base.owner_attached);
        assert_eq!(person.base.received_time, 123);
        assert_eq!(person.received.effects.effects()[0].time_70, 0x10007);
        assert_eq!(person.received.controllers.entries().len(), 2);
        assert_eq!(person.body.creature.unit.fields.field_3a8, Some(-7));
        assert_eq!(person.body.creature.body.as_ref().unwrap().hit_points, -30);
        assert_eq!(
            person.body.sentient.skills.as_ref().unwrap().0[&0x8000],
            i32::MIN
        );
        assert_eq!(
            person.body.sentient.orders.as_ref().unwrap()[0].position,
            [1., 2., 3.]
        );
        assert_eq!(person.body.person.field_1408, Some(1.25));
        assert_eq!(person.body.person.field_83d, Some(11));
        assert_eq!(h.lookup.find_atom(7), Some(90));
        assert_eq!(h.owner.assigned, [90]);
        assert!(!h.connector.0.is_empty());
        assert!(h.connector.0.iter().all(|(handle, _)| *handle == 90));
        assert_eq!(h.hooks.live[&90].hp, -30);
        assert_eq!(
            h.hooks.reports[0]
                .1
                .person
                .inventory
                .as_ref()
                .unwrap()
                .null_factories,
            [(0, 0), (1, 21)]
        );
        let signal_count = h.connector.0.len();
        let second_bytes = packet(456, &[0, 0, 0, 0, 0xab, 0, 0, 0xee]);
        let second = h.receive(&second_bytes).unwrap().unwrap();
        assert!(!second.is_new);
        assert_eq!(second.handle, 90);
        assert_eq!(second.consumed_bytes, second_bytes.len() - 1);
        assert_eq!(h.people.len(), 1);
        assert!(h.people.get(91).is_none());
        assert!(h.graph.node(91).is_none());
        assert!(!h.hooks.live.contains_key(&91));
        assert_eq!(h.people.get(90).unwrap().base.received_time, 456);
        assert_eq!(h.people.get(90).unwrap().body.person.field_83d, Some(11));
        assert_eq!(h.connector.0.len(), signal_count);
        assert_eq!(h.owner.assigned, [90]);
        assert_eq!(
            h.hooks.events,
            [
                (90, "construct"),
                (90, "body"),
                (90, "report"),
                (90, "post"),
                (91, "construct"),
                (91, "destroy"),
                (90, "body"),
                (90, "report"),
                (90, "post")
            ]
        );
        assert_eq!(h.hooks.post, [(90, true), (90, false)]);
    }
}
