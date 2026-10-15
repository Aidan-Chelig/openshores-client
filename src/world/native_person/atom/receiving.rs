use super::{NativeBaseReceiveOutcome, NativePersonObjects};
use crate::{
    protocol::QtReader,
    world::{
        native_atom::NativeAtomKind,
        native_subatoms::{NativeNormalSubatoms, NativeRecoveryLookup, NativeRetiredAtoms},
    },
};
use std::{io, time::Instant};

pub struct NativePersonCandidateReceiveContext<'a> {
    pub graph: &'a mut NativeNormalSubatoms,
    pub lookup: &'a NativeRecoveryLookup,
    pub retired: &'a NativeRetiredAtoms,
    pub position_epoch: Instant,
    pub profile: crate::atom_position::FloatEncoding,
}
impl NativePersonObjects {
    /// Receive base layers before fresh Person owner addition. Position/status
    /// changes update both concrete Person and cache graph in native stage order.
    /// Auxiliary callback must complete the conditional offset-0xa0 reset.
    /// Later errors retain earlier updates; no rollback or implicit owner add.
    pub fn receive_candidate_base(
        &mut self,
        handle: u32,
        input: &mut QtReader<'_>,
        context: NativePersonCandidateReceiveContext<'_>,
        mut position_auxiliary: impl FnMut(u32) -> io::Result<()>,
        clock: impl FnMut() -> (i64, i64),
    ) -> io::Result<NativeBaseReceiveOutcome> {
        let person = self.get_mut(handle).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "unknown native Person handle")
        })?;
        context.graph.validate_unowned_candidate(
            handle,
            person.base.owner_attached,
            context.lookup,
            context.retired,
        )?;
        if context
            .graph
            .node(handle)
            .expect("validated candidate")
            .kind
            != NativeAtomKind::Person
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Person storage/cache class mismatch",
            ));
        }
        let result = person.receive_base(
            input,
            context.profile,
            |base, record| {
                context.graph.apply_candidate_position_at(
                    handle,
                    &record,
                    context.position_epoch,
                    false,
                    context.lookup,
                    context.retired,
                )?;
                base.position = context
                    .graph
                    .node(handle)
                    .expect("known candidate")
                    .position
                    .clone();
                position_auxiliary(handle)
            },
            clock,
        );
        // Status reset precedes later fields. Even a later decode error must
        // leave its completed write reflected in the graph.
        if person.base.field_1c == 0 {
            context.graph.finish_received_position_stage(handle)?;
        }
        result
    }
}
