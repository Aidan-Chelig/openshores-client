use super::base::{NativeBaseReceiveAdapter, NativeBaseReceiveFields};
use super::*;
use crate::{
    atom_position::{FloatEncoding, PackedPosition},
    world::{native_atom::NativeBaseAtomInitialization, native_item::NativeItemInitialization},
};

struct Context {
    fields: NativeBaseReceiveFields,
    lookup: NativeRecoveryLookup,
    events: Vec<String>,
    item: NativeItemInitialization,
}
impl NativeAtomDispatchContext for Context {
    fn lookup(&self) -> &NativeRecoveryLookup {
        &self.lookup
    }
    fn construct_candidate(&mut self, kind: NativeAtomKind) -> io::Result<u32> {
        self.events.push(format!("construct:{kind:?}"));
        Ok(90)
    }
    fn assign_identity(&mut self, handle: u32, identity: NativeIdentityPrefix) -> io::Result<()> {
        self.events
            .push(format!("identity:{handle}:{}", identity.id));
        Ok(())
    }
    fn assign_time(&mut self, handle: u32, timestamp_ms: i64) -> io::Result<()> {
        self.events.push(format!("time:{handle}"));
        self.item.base.received_time = timestamp_ms;
        Ok(())
    }
    fn receive_base(
        &mut self,
        handle: u32,
        _identity: NativeIdentityPrefix,
        input: &mut QtReader<'_>,
    ) -> io::Result<()> {
        self.events.push(format!("base:{handle}"));
        let mut adapter = NativeBaseReceiveAdapter {
            atom: &mut self.item.base,
            fields: &mut self.fields,
            position: |atom: &mut NativeBaseAtomInitialization, record: PackedPosition| {
                atom.position.apply_fresh_wire_position(&record, false)?;
                Ok(())
            },
            clock: || (0x10000, 0),
        };
        base::receive_base(input, FloatEncoding::TypeWidths, &mut adapter)?;
        Ok(())
    }
    fn connect_signals(&mut self, handle: u32) -> io::Result<()> {
        self.events.push(format!("signals:{handle}"));
        Ok(())
    }
    fn add_atom(&mut self, handle: u32) -> io::Result<()> {
        self.events.push(format!("add:{handle}"));
        Ok(())
    }
    fn destroy_candidate(&mut self, handle: u32) -> io::Result<()> {
        self.events.push(format!("destroy:{handle}"));
        Ok(())
    }
    fn receive_body(
        &mut self,
        handle: u32,
        is_new: bool,
        input: &mut QtReader<'_>,
    ) -> io::Result<()> {
        self.events.push(format!("body:{handle}:{is_new}"));
        // This fixture's selected object is Item, even for a Person candidate.
        let body = crate::world::native_item::NativeItemBody::read(input)?.unwrap();
        self.item.state.apply(body);
        Ok(())
    }
    fn post_update(&mut self, handle: u32, is_new: bool) -> io::Result<()> {
        self.events.push(format!("post:{handle}:{is_new}"));
        Ok(())
    }
}
fn context() -> Context {
    Context {
        fields: Default::default(),
        lookup: NativeRecoveryLookup::new([], []).unwrap(),
        events: vec![],
        item: NativeItemInitialization::from_default_item_constructor(),
    }
}
fn payload(opcode: u8) -> Vec<u8> {
    let mut bytes = vec![opcode];
    bytes.extend(7_u32.to_be_bytes());
    bytes.extend((-123_i64).to_be_bytes());
    bytes.extend([0, 0x60, 0xde, 0xad]);
    bytes
}
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
