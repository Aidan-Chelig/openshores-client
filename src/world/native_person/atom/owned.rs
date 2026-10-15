use super::{NativeBaseReceiveOutcome, NativePersonObjects};
use crate::{
    atom_position::PackedPosition,
    protocol::QtReader,
    world::{
        native_atom::NativeAtomKind,
        native_position_updates::{
            DestinationContext, NativePositionReceiver, NativePositionUpdate,
        },
        native_subatoms::{
            NativeRecoveryLookup, NativeReparentResult, ReparentRegistrationContext,
        },
    },
};
use std::{io, time::Instant};

pub struct NativePersonOwnedReceiveContext<'a, 'r, O> {
    pub receiver: &'a mut NativePositionReceiver<'r, O>,
    pub lookup: &'a NativeRecoveryLookup,
    pub position_epoch: Instant,
    pub profile: crate::atom_position::FloatEncoding,
}
impl NativePersonObjects {
    /// Retained owned Person base reception. Explicit destination observations,
    /// registration context, auxiliary position completion and clocks are
    /// required. Deferred owner obligations stop subsequent layers with error.
    pub fn receive_owned_base<O: ReparentRegistrationContext>(
        &mut self,
        handle: u32,
        input: &mut QtReader<'_>,
        context: NativePersonOwnedReceiveContext<'_, '_, O>,
        mut destination: impl FnMut(&PackedPosition) -> DestinationContext,
        mut position_auxiliary: impl FnMut(u32) -> io::Result<()>,
        clock: impl FnMut() -> (i64, i64),
    ) -> io::Result<NativeBaseReceiveOutcome> {
        let person = self.get_mut(handle).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "unknown retained Person handle",
            )
        })?;
        if !person.base.owner_attached
            || context.lookup.find_atom(person.base.wire_id) != Some(handle)
            || context
                .receiver
                .graph
                .node(handle)
                .is_none_or(|node| node.kind != NativeAtomKind::Person)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "retained Person owner/lookup/class context is inconsistent",
            ));
        }
        let id = person.base.wire_id;
        let result = person.receive_base(
            input,
            context.profile,
            |base, record| {
                let outcome = context.receiver.apply_record_resolved_at(
                    id,
                    handle,
                    &record,
                    destination(&record),
                    context.position_epoch,
                );
                // Callback failures may already have moved indexes/position/timers.
                base.position = context
                    .receiver
                    .graph
                    .node(handle)
                    .expect("known retained Person")
                    .position
                    .clone();
                match outcome? {
                    NativePositionUpdate::ReparentRequired(_)
                    | NativePositionUpdate::NormalOwner(
                        NativeReparentResult::OwnerContextRequired { .. },
                    ) => Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "native Person position requires unresolved owner context",
                    )),
                    _ => position_auxiliary(handle),
                }
            },
            clock,
        );
        if person.base.field_1c == 0 {
            context
                .receiver
                .graph
                .finish_received_position_stage(handle)?;
        }
        result
    }
}
