use super::*;
use crate::world::{
    native_atom::NativeAtomKind,
    native_prefix::NativeIdentityPrefix,
    native_subatoms::{NativeNormalSubatoms, NativeRecoveryLookup, NativeRetiredAtoms},
};
use std::time::Instant;
mod lifecycle;

#[test]
fn same_wire_identity_never_aliases_candidate_storage_or_disposes_retained_person() {
    let mut graph = NativeNormalSubatoms::new([], []).unwrap();
    let mut people = NativePersonObjects::default();
    for handle in [10, 90] {
        people
            .construct_candidate(NativeAtomKind::Person, handle, &mut graph)
            .unwrap();
        people
            .get_mut(handle)
            .unwrap()
            .base
            .apply_received_identity(NativeIdentityPrefix {
                id: 7,
                timestamp_ms: 123,
            });
    }
    people.get_mut(10).unwrap().body.person.field_f80 = Some(99);
    assert_eq!(people.get(90).unwrap().body.person.field_f80, Some(0));
    let lookup = NativeRecoveryLookup::new([(7, 10)], []).unwrap();
    let retired = NativeRetiredAtoms::default();
    assert!(
        people
            .discard_candidate(10, &mut graph, &lookup, &retired)
            .is_err()
    );
    people
        .discard_candidate(90, &mut graph, &lookup, &retired)
        .unwrap();
    assert_eq!(people.len(), 1);
    assert!(graph.node(90).is_none());
    assert_eq!(lookup.find_atom(7), Some(10));
    assert_eq!(people.get(10).unwrap().body.person.field_f80, Some(99));
}

#[test]
fn invalid_construction_and_graph_collisions_do_not_admit_person_storage() {
    let mut graph = NativeNormalSubatoms::new([], []).unwrap();
    let mut people = NativePersonObjects::default();
    assert_eq!(
        people
            .construct_candidate(NativeAtomKind::Item, 90, &mut graph)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert!(people.is_empty());
    people
        .construct_candidate(NativeAtomKind::Person, 90, &mut graph)
        .unwrap();
    assert!(
        people
            .construct_candidate(NativeAtomKind::Person, 90, &mut graph)
            .is_err()
    );
    let mut other = NativePersonObjects::default();
    assert!(
        other
            .construct_candidate(NativeAtomKind::Person, 90, &mut graph)
            .is_err()
    );
    assert!(other.is_empty());
    assert_eq!(people.len(), 1);
    assert_eq!(graph.node(90).unwrap().field_1c, 5);
    let mut wrong_class = graph.node(90).unwrap().clone();
    wrong_class.kind = NativeAtomKind::Item;
    let mut wrong_graph = NativeNormalSubatoms::new([(90, wrong_class)], []).unwrap();
    let lookup = NativeRecoveryLookup::new([], []).unwrap();
    let retired = NativeRetiredAtoms::default();
    let mut input = QtReader::new(&[0, 0xee]);
    assert_eq!(
        people
            .receive_candidate_base(
                90,
                &mut input,
                NativePersonCandidateReceiveContext {
                    graph: &mut wrong_graph,
                    lookup: &lookup,
                    retired: &retired,
                    position_epoch: Instant::now(),
                    profile: FloatEncoding::TypeWidths,
                },
                |_| panic!("invalid class"),
                || panic!("invalid class")
            )
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(input.remaining_bytes(), &[0, 0xee]);
    assert_eq!(people.get(90).unwrap().base.field_1c, 5);
}

#[test]
fn candidate_position_and_status_survive_later_base_failure_for_every_float_profile() {
    for profile in [
        FloatEncoding::TypeWidths,
        FloatEncoding::SinglePrecision,
        FloatEncoding::DoublePrecision,
    ] {
        let mut graph = NativeNormalSubatoms::new([], []).unwrap();
        let mut people = NativePersonObjects::default();
        people
            .construct_candidate(NativeAtomKind::Person, 90, &mut graph)
            .unwrap();
        let lookup = NativeRecoveryLookup::new([], []).unwrap();
        let retired = NativeRetiredAtoms::default();
        let mut bytes = vec![5, 0x83];
        bytes.extend(12_u32.to_be_bytes());
        if profile == FloatEncoding::SinglePrecision {
            bytes.extend(12.5_f32.to_be_bytes());
        } else {
            bytes.extend(12.5_f64.to_be_bytes());
        }
        bytes.push(0);
        let mut input = QtReader::new(&bytes);
        let mut aux = 0;
        assert_eq!(
            people
                .receive_candidate_base(
                    90,
                    &mut input,
                    NativePersonCandidateReceiveContext {
                        graph: &mut graph,
                        lookup: &lookup,
                        retired: &retired,
                        position_epoch: Instant::now(),
                        profile,
                    },
                    |_| {
                        aux += 1;
                        Ok(())
                    },
                    || panic!("unreached effects")
                )
                .unwrap_err()
                .kind(),
            io::ErrorKind::UnexpectedEof
        );
        let person = people.get(90).unwrap();
        let node = graph.node(90).unwrap();
        assert_eq!(aux, 1);
        assert_eq!(person.base.field_1c, 0);
        assert_eq!(node.field_1c, 0);
        assert_eq!(person.base.position, node.position);
        assert_eq!(node.position.position.parent, 12);
        assert_eq!(node.position.position.local[3][0], 12.5);
        assert_eq!(input.remaining_bytes(), &[0]);
        assert_eq!(person.received.modified_ms, None);
        assert!(!person.base.owner_attached);
    }
}
