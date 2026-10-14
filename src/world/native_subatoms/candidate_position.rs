use super::*;
use crate::atom_position::{PackedPosition, PositionChange};
use std::time::Instant;

impl NativeNormalSubatoms {
    /// Fresh candidate's unowned ChangePos branch, before AddAtom establishes
    /// owner indexes. Parent changes are local copies, not managed reparenting.
    /// Explicit absence of owner/map/list/retention references is required.
    /// Does not perform the later base status write or conditional 0xa0 reset.
    pub fn apply_candidate_position_at(
        &mut self,
        handle: u32,
        record: &PackedPosition,
        now: Instant,
        has_owner: bool,
        lookup: &NativeRecoveryLookup,
        retired: &NativeRetiredAtoms,
    ) -> io::Result<PositionChange> {
        self.check_unowned_candidate(handle, has_owner, lookup, retired)?;
        self.nodes
            .get_mut(&handle)
            .expect("validated candidate")
            .position
            .apply_fresh_wire_position_at(record, false, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        atom_position::FloatEncoding, protocol::QtReader,
        world::native_item::NativeItemInitialization,
    };
    #[test]
    fn unowned_parent_update_copies_position_before_owner_and_status_establishment() {
        let item = NativeItemInitialization::from_default_item_constructor();
        let mut graph = NativeNormalSubatoms::new([], []).unwrap();
        graph
            .insert_candidate(
                90,
                NativeCacheNode {
                    kind: NativeAtomKind::Item,
                    field_1c: item.base.field_1c,
                    position: item.base.position,
                },
            )
            .unwrap();
        let mut bytes = vec![0x83];
        bytes.extend(12_u32.to_be_bytes());
        bytes.extend(12345678.125_f64.to_be_bytes());
        let record =
            PackedPosition::read(&mut QtReader::new(&bytes), FloatEncoding::TypeWidths).unwrap();
        let lookup = NativeRecoveryLookup::new([], []).unwrap();
        let retired = NativeRetiredAtoms::default();
        let now = Instant::now();
        assert!(matches!(
            graph
                .apply_candidate_position_at(90, &record, now, false, &lookup, &retired)
                .unwrap(),
            PositionChange::Applied { .. }
        ));
        let node = graph.node(90).unwrap();
        assert_eq!(node.position.position.parent, 12);
        assert_eq!(node.position.position.local[3][0], 12345678.125);
        assert_eq!(node.position.orientation, Some([0.; 3]));
        assert_eq!(node.field_1c, 5);
        assert!(graph.lists.is_empty());
        let before = node.clone();
        let indexed = NativeRecoveryLookup::new([(7, 90)], []).unwrap();
        assert!(
            graph
                .apply_candidate_position_at(90, &record, now, false, &indexed, &retired)
                .is_err()
        );
        assert_eq!(graph.node(90), Some(&before));
        assert!(
            graph
                .apply_candidate_position_at(90, &record, now, true, &lookup, &retired)
                .is_err()
        );
        assert_eq!(graph.node(90), Some(&before));
    }
}
