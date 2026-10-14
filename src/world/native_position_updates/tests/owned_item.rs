use super::*;
use crate::world::{
    native_item::{NativeItemObjects, NativeItemOwnedReceiveContext},
    native_prefix::NativeIdentityPrefix,
    native_subatoms::NativeRecoveryLookup,
};

#[test]
fn retained_item_receive_uses_real_missing_parent_queue_and_registration() {
    let mut graph = NativeNormalSubatoms::new(
        [],
        [(
            7,
            NativeChildList {
                available: true,
                children: vec![],
            },
        )],
    )
    .unwrap();
    let mut items = NativeItemObjects::default();
    items
        .construct_candidate(NativeAtomKind::Item, 90, &mut graph)
        .unwrap();
    let item = items.get_mut(90).unwrap();
    item.base.apply_received_identity(NativeIdentityPrefix {
        id: 7,
        timestamp_ms: 123,
    });
    item.base.owner_attached = true;
    item.base.finish_received_position_stage();
    graph.finish_received_position_stage(90).unwrap();
    let lookup = NativeRecoveryLookup::new([(7, 90)], []).unwrap();
    let mut queue = NativeRunaways::default();
    let mut registrations = RegistrationStore::default();
    let mut owner = Owner::default();
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut registrations,
        owner: &mut owner,
    };
    let mut bytes = vec![1, 0x83];
    bytes.extend(12_u32.to_be_bytes());
    bytes.extend(12.5_f64.to_be_bytes());
    bytes.push(0xde);
    let mut input = crate::protocol::QtReader::new(&bytes);
    let mut aux = 0;
    items
        .receive_owned_base(
            90,
            &mut input,
            NativeItemOwnedReceiveContext {
                receiver: &mut receiver,
                lookup: &lookup,
                position_epoch: Instant::now(),
            },
            |_| DestinationContext::MissingDestinationLock,
            |_| {
                aux += 1;
                Ok(())
            },
            || panic!("no effects"),
        )
        .unwrap();
    assert_eq!(aux, 1);
    assert_eq!(input.remaining_bytes(), &[0xde]);
    assert!(receiver.runaways.contains(12, 7));
    assert!(!receiver.runaways.contains(12, 90));
    assert!(receiver.registrations.ticker(7).is_some());
    assert!(receiver.registrations.ticker(90).is_none());
    assert_eq!(
        items.get(90).unwrap().base.position,
        receiver.graph.node(90).unwrap().position
    );
    assert_eq!(items.get(90).unwrap().base.position.position.parent, 12);
}
