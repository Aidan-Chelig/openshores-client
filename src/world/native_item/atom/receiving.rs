use super::{NativeBaseReceiveOutcome, NativeItemObjects};
use crate::{
    protocol::QtReader,
    world::{
        native_atom::NativeAtomKind,
        native_subatoms::{NativeNormalSubatoms, NativeRecoveryLookup, NativeRetiredAtoms},
    },
};
use std::{io, time::Instant};

pub struct NativeItemCandidateReceiveContext<'a> {
    pub graph: &'a mut NativeNormalSubatoms,
    pub lookup: &'a NativeRecoveryLookup,
    pub retired: &'a NativeRetiredAtoms,
    pub position_epoch: Instant,
}
impl NativeItemObjects {
    /// Receive base layers before fresh Item owner addition. Position/status
    /// changes update both concrete Item and cache graph in native stage order.
    /// Auxiliary callback must complete the conditional offset-0xa0 reset.
    /// Later errors retain earlier updates; no rollback or implicit owner add.
    pub fn receive_candidate_base(
        &mut self,
        handle: u32,
        input: &mut QtReader<'_>,
        context: NativeItemCandidateReceiveContext<'_>,
        mut position_auxiliary: impl FnMut(u32) -> io::Result<()>,
        clock: impl FnMut() -> (i64, i64),
    ) -> io::Result<NativeBaseReceiveOutcome> {
        let item = self.get_mut(handle).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "unknown native Item handle")
        })?;
        context.graph.validate_unowned_candidate(
            handle,
            item.base.owner_attached,
            context.lookup,
            context.retired,
        )?;
        if context
            .graph
            .node(handle)
            .expect("validated candidate")
            .kind
            != NativeAtomKind::Item
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Item storage/cache class mismatch",
            ));
        }
        let result = item.receive_base(
            input,
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
        if item.base.field_1c == 0 {
            context.graph.finish_received_position_stage(handle)?;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn candidate_receive_keeps_graph_and_item_after_later_time_failure() {
        let mut graph = NativeNormalSubatoms::new([], []).unwrap();
        let mut objects = NativeItemObjects::default();
        objects
            .construct_candidate(NativeAtomKind::Item, 90, &mut graph)
            .unwrap();
        let lookup = NativeRecoveryLookup::new([], []).unwrap();
        let retired = NativeRetiredAtoms::default();
        let mut bytes = vec![5, 0x83];
        bytes.extend(12_u32.to_be_bytes());
        bytes.extend(12345678.125_f64.to_be_bytes());
        bytes.push(0);
        let mut input = QtReader::new(&bytes);
        let mut aux = 0;
        assert!(
            objects
                .receive_candidate_base(
                    90,
                    &mut input,
                    NativeItemCandidateReceiveContext {
                        graph: &mut graph,
                        lookup: &lookup,
                        retired: &retired,
                        position_epoch: Instant::now()
                    },
                    |handle| {
                        assert_eq!(handle, 90);
                        aux += 1;
                        Ok(())
                    },
                    || panic!("unreached effects")
                )
                .is_err()
        );
        let item = objects.get(90).unwrap();
        let node = graph.node(90).unwrap();
        assert_eq!(aux, 1);
        assert_eq!(item.base.field_1c, 0);
        assert_eq!(node.field_1c, 0);
        assert_eq!(item.base.position, node.position);
        assert_eq!(node.position.position.parent, 12);
        assert_eq!(node.position.position.local[3][0], 12345678.125);
        assert_eq!(input.remaining_bytes(), &[0]);
        assert_eq!(item.received.modified_ms, None);
    }
}
