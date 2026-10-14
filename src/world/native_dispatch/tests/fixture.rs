use super::base::{NativeBaseReceiveAdapter, NativeBaseReceiveFields};
use super::*;
use crate::{
    atom_position::{FloatEncoding, PackedPosition},
    world::{native_atom::NativeBaseAtomInitialization, native_item::NativeItemInitialization},
};

pub(super) struct Context {
    pub(super) signal_bindings: Vec<crate::world::native_signals::NativeSignalConnection>,
    pub(super) signal_mode: i32,
    pub(super) accept_connections: bool,
    pub(super) fields: NativeBaseReceiveFields,
    pub(super) lookup: NativeRecoveryLookup,
    pub(super) events: Vec<String>,
    pub(super) item: NativeItemInitialization,
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
        crate::world::native_signals::connect_signals(handle, self.signal_mode, self)?;
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
impl crate::world::native_signals::NativeSignalConnector for Context {
    fn connect(
        &mut self,
        handle: u32,
        binding: crate::world::native_signals::NativeSignalConnection,
    ) -> io::Result<bool> {
        assert_eq!(handle, 90);
        self.signal_bindings.push(binding);
        Ok(self.accept_connections)
    }
}
pub(super) fn context() -> Context {
    Context {
        signal_bindings: vec![],
        signal_mode: 5, // Explicit fixture mode, not inferred from source login.
        accept_connections: true,
        fields: Default::default(),
        lookup: NativeRecoveryLookup::new([], []).unwrap(),
        events: vec![],
        item: NativeItemInitialization::from_default_item_constructor(),
    }
}
pub(super) fn payload(opcode: u8) -> Vec<u8> {
    let mut bytes = vec![opcode];
    bytes.extend(7_u32.to_be_bytes());
    bytes.extend((-123_i64).to_be_bytes());
    bytes.extend([0, 0x60, 0xde, 0xad]);
    bytes
}
