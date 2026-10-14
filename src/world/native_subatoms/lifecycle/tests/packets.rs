use super::*;
use crate::{
    atom_position::PackedPosition,
    world::{
        native_dispatch::{
            item::{NativeItemDispatch, NativeItemDispatchHooks},
            receive_atom,
        },
        native_item::NativeItemObjects,
        native_position_updates::DestinationContext,
        native_signals::{NativeSignalConnection, NativeSignalConnector},
    },
};

struct Hooks {
    next: u32,
    post: Vec<(u32, bool)>,
}
impl NativeItemDispatchHooks for Hooks {
    fn allocate_handle(&mut self, kind: NativeAtomKind) -> io::Result<u32> {
        assert_eq!(kind, NativeAtomKind::Item);
        let handle = self.next;
        self.next += 1;
        Ok(handle)
    }
    fn position_epoch(&mut self) -> Instant {
        Instant::now()
    }
    fn destination(&mut self, _: u32, _: &PackedPosition) -> DestinationContext {
        panic!("omitted position")
    }
    fn position_auxiliary(&mut self, _: u32) -> io::Result<()> {
        panic!("omitted position")
    }
    fn effect_clock(&mut self) -> (i64, i64) {
        panic!("omitted effects")
    }
    fn post_update(&mut self, handle: u32, is_new: bool) -> io::Result<()> {
        self.post.push((handle, is_new));
        Ok(())
    }
}
#[derive(Default)]
struct Connector {
    bindings: Vec<NativeSignalConnection>,
}
impl NativeSignalConnector for Connector {
    fn connect(&mut self, handle: u32, binding: NativeSignalConnection) -> io::Result<bool> {
        assert_eq!(handle, 71);
        self.bindings.push(binding);
        Ok(false)
    }
}
fn packet(time: i64, body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x16];
    bytes.extend(7_u32.to_be_bytes());
    bytes.extend(time.to_be_bytes());
    bytes.push(0);
    bytes.extend(body);
    bytes
}
#[test]
fn concrete_packet_dispatch_constructs_adds_reuses_and_disposes_real_item_storage() {
    let mut graph = NativeNormalSubatoms::new([], []).unwrap();
    let mut items = NativeItemObjects::default();
    let mut lookup = NativeRecoveryLookup::new([], []).unwrap();
    let mut queue = NativeRunaways::default();
    let mut registrations = RegistrationStore::default();
    let mut retired = NativeRetiredAtoms::default();
    let events = Rc::new(RefCell::new(vec![]));
    let mut owner = Owner {
        events: events.clone(),
        owners: BTreeMap::from([(71, false)]),
    };
    let mut connector = Connector::default();
    let mut hooks = Hooks {
        next: 71,
        post: vec![],
    };
    let mut dispatch = NativeItemDispatch {
        items: &mut items,
        graph: &mut graph,
        lookup: &mut lookup,
        runaways: &mut queue,
        registrations: &mut registrations,
        retired: &mut retired,
        owner: &mut owner,
        connector: &mut connector,
        hooks: &mut hooks,
        raw_mode: 5,
    };
    let first = receive_atom(&packet(123, &[0x60, 0xde]), &mut dispatch)
        .unwrap()
        .unwrap();
    assert!(first.is_new);
    assert_eq!(first.handle, 71);
    assert_eq!(first.consumed_bytes, 15);
    assert_eq!(dispatch.lookup.find_atom(7), Some(71));
    assert!(dispatch.items.get(71).unwrap().base.owner_attached);
    assert_eq!(dispatch.items.get(71).unwrap().item.ownership_id, Some(1));
    assert_eq!(dispatch.connector.bindings.len(), 1); // First wiring rejection doesn't abort packet.
    let second = receive_atom(&packet(-456, &[0]), &mut dispatch)
        .unwrap()
        .unwrap();
    assert!(!second.is_new);
    assert_eq!(second.handle, 71);
    assert_eq!(dispatch.items.len(), 1);
    assert!(dispatch.graph.node(72).is_none());
    assert_eq!(dispatch.items.get(71).unwrap().base.received_time, -456);
    assert_eq!(dispatch.items.get(71).unwrap().item.ownership_id, Some(1));
    assert_eq!(dispatch.hooks.post, [(71, true), (71, false)]);
    assert!(dispatch.retired.is_empty());
    assert_eq!(*events.borrow(), ["assign"]);
    assert_eq!(
        receive_atom(&packet(789, &[1, 249]), &mut dispatch)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(dispatch.items.len(), 1);
    assert!(dispatch.graph.node(73).is_none());
    assert_eq!(dispatch.items.get(71).unwrap().base.received_time, 789);
    assert_eq!(dispatch.hooks.post.len(), 2);
    assert_eq!(dispatch.lookup.find_atom(7), Some(71));
}
