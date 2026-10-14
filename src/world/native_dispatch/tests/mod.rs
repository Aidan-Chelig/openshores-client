use super::*;
mod fixture;
use fixture::{context, payload};

#[test]
fn dispatch_crosses_real_position_effect_controller_layers_into_retained_item_body() {
    let mut bytes = payload(0x05)[..13].to_vec();
    bytes.extend([0x71, 0x82]);
    bytes.extend(12345678.125_f64.to_be_bytes());
    bytes.extend([1, 0, 0, 7, 2, 2, 2, 0x60, 0xde, 0xad]);
    let mut context = context();
    context.lookup = NativeRecoveryLookup::new([(7, 10)], []).unwrap();
    let outcome = receive_atom(&bytes, &mut context).unwrap().unwrap();
    assert_eq!(outcome.handle, 10);
    assert!(!outcome.is_new);
    assert_eq!(outcome.consumed_bytes, bytes.len() - 2);
    assert_eq!(
        context.item.base.position.position.local[3][0],
        12345678.125
    );
    assert_eq!(context.fields.effects.effects()[0].time_70, 0x10007);
    assert_eq!(context.fields.controllers.entries().len(), 2);
    assert_eq!(context.fields.in_motion, Some(true));
    assert_eq!(context.item.state.ownership_id, Some(1));
    assert!(context.item.state.field_39c.unwrap());
    assert_eq!(&bytes[outcome.consumed_bytes..], &[0xde, 0xad]);
    assert_eq!(
        &context.events[4..],
        &["destroy:90", "body:10:false", "post:10:false"]
    );
}

#[test]
fn existing_primary_item_receives_person_candidate_without_replacement() {
    let mut context = context();
    context.lookup = NativeRecoveryLookup::new([(7, 10)], [(7, 20)]).unwrap();
    let result = receive_atom(&payload(0x05), &mut context).unwrap().unwrap();
    assert_eq!(
        result,
        NativeAtomDispatchOutcome {
            wire_id: 7,
            handle: 10,
            is_new: false,
            consumed_bytes: 15
        }
    );
    assert_eq!(
        context.events,
        [
            "construct:Person",
            "identity:90:7",
            "time:10",
            "base:10",
            "destroy:90",
            "body:10:false",
            "post:10:false"
        ]
    );
    assert_eq!(context.item.base.received_time, -123);
    assert_eq!(context.item.base.field_1c, 0);
    assert_eq!(context.item.state.ownership_id, Some(1));
}
#[test]
fn fresh_item_is_connected_and_added_before_body() {
    let mut context = context();
    let result = receive_atom(&payload(0x16), &mut context).unwrap().unwrap();
    assert!(result.is_new);
    assert_eq!(
        context.signal_bindings,
        crate::world::native_signals::connection_plan(5)
    );
    assert_eq!(
        context.events,
        [
            "construct:Item",
            "identity:90:7",
            "base:90",
            "signals:90",
            "add:90",
            "body:90:true",
            "post:90:true"
        ]
    );
}
#[test]
fn first_signal_rejection_does_not_abort_fresh_packet_owner_or_body() {
    let mut context = context();
    context.accept_connections = false;
    let outcome = receive_atom(&payload(0x16), &mut context).unwrap().unwrap();
    assert!(outcome.is_new);
    assert_eq!(context.signal_bindings.len(), 1);
    assert_eq!(
        &context.events[3..],
        &["signals:90", "add:90", "body:90:true", "post:90:true"]
    );
    assert_eq!(context.item.state.ownership_id, Some(1));
}
#[test]
fn malformed_base_preserves_earlier_time_and_stops_later_operations() {
    let mut context = context();
    context.lookup = NativeRecoveryLookup::new([], [(7, 20)]).unwrap();
    assert!(receive_atom(&payload(0x05)[..13], &mut context).is_err());
    assert_eq!(
        context.events,
        ["construct:Person", "identity:90:7", "time:20", "base:20"]
    );
    assert_eq!(context.item.base.received_time, -123);
    assert_eq!(context.item.base.field_1c, 5);
}
#[test]
fn unknown_opcode_has_no_context_effects() {
    let mut context = context();
    assert!(receive_atom(&[0xff], &mut context).unwrap().is_none());
    assert!(context.events.is_empty());
}
