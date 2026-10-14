use super::{NativeBaseReceiveOutcome, NativeItemObjects};
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

pub struct NativeItemOwnedReceiveContext<'a, 'r, O> {
    pub receiver: &'a mut NativePositionReceiver<'r, O>,
    pub lookup: &'a NativeRecoveryLookup,
    pub position_epoch: Instant,
}
impl NativeItemObjects {
    /// Retained owned Item base reception. Explicit destination observations,
    /// registration context, auxiliary position completion and clocks are
    /// required. Deferred owner obligations stop subsequent layers with error.
    pub fn receive_owned_base<O: ReparentRegistrationContext>(
        &mut self,
        handle: u32,
        input: &mut QtReader<'_>,
        context: NativeItemOwnedReceiveContext<'_, '_, O>,
        mut destination: impl FnMut(&PackedPosition) -> DestinationContext,
        mut position_auxiliary: impl FnMut(u32) -> io::Result<()>,
        clock: impl FnMut() -> (i64, i64),
    ) -> io::Result<NativeBaseReceiveOutcome> {
        let item = self.get_mut(handle).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "unknown retained Item handle")
        })?;
        if !item.base.owner_attached
            || context.lookup.find_atom(item.base.wire_id) != Some(handle)
            || context
                .receiver
                .graph
                .node(handle)
                .is_none_or(|node| node.kind != NativeAtomKind::Item)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "retained Item owner/lookup/class context is inconsistent",
            ));
        }
        let id = item.base.wire_id;
        let result = item.receive_base(
            input,
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
                    .expect("known retained Item")
                    .position
                    .clone();
                match outcome? {
                    NativePositionUpdate::ReparentRequired(_)
                    | NativePositionUpdate::NormalOwner(
                        NativeReparentResult::OwnerContextRequired { .. },
                    ) => Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "native Item position requires unresolved owner context",
                    )),
                    _ => position_auxiliary(handle),
                }
            },
            clock,
        );
        if item.base.field_1c == 0 {
            context
                .receiver
                .graph
                .finish_received_position_stage(handle)?;
        }
        result
    }
}
