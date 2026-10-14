use super::*;
use crate::world::native_subatoms::NativeRecoveryLookup;

#[test]
fn resolved_replay_uses_map_handles_but_reports_wire_ids_and_resolution_failure_frame() {
    let capture = capture(&[
        packet(1, 0, 12.5, 1, true),
        packet(1, 0, 99., 2, false),
        packet(2, 0, 33., 3, true),
        packet(1, 0, 44., 4, true),
    ]);
    let payloads = capture
        .frames
        .iter()
        .map(|f| f.payload().to_vec())
        .collect::<Vec<_>>();
    let mut graph = NativeNormalSubatoms::new(
        [(
            10,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: 0,
                position: NativePositionState::default(),
            },
        )],
        [],
    )
    .unwrap();
    let lookup = NativeRecoveryLookup::from_routed_nodes([(1, 10, NativeAtomKind::Item)]).unwrap();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner;
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    let now = Instant::now();
    let mut evaluated = Vec::new();
    let error = capture
        .replay_resolved_positions(&mut receiver, |frame, prefix| {
            evaluated.push(frame);
            let handle = lookup
                .routed_entry(prefix.base.id, prefix.kind)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "fixture lookup failed"))?;
            Ok((handle, DestinationContext::Unresolved, now))
        })
        .unwrap_err();
    assert_eq!(error.frame, Some(2));
    assert_eq!(error.error.kind(), io::ErrorKind::NotFound);
    assert_eq!(error.completed.positions.len(), 1);
    assert_eq!(error.completed.positions[0].id, 1);
    assert_eq!(error.completed.positions[0].frame, 0);
    assert_eq!(error.completed.no_position_frames, 1);
    assert_eq!(evaluated, [0, 2]);
    assert_eq!(graph.node(10).unwrap().position.position.local[3][0], 12.5);
    assert!(graph.node(1).is_none());
    assert!(queue.is_empty());
    assert!(store.ticker(1).is_none());
    assert_eq!(
        capture
            .frames
            .iter()
            .map(|f| f.payload().to_vec())
            .collect::<Vec<_>>(),
        payloads
    );
}

#[test]
fn resolved_replay_reports_unresolved_reparent_without_changing_aliased_object() {
    let capture = capture(&[packet(1, 7, 12.5, 1, true)]);
    let mut graph = NativeNormalSubatoms::new(
        [(
            10,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: 0,
                position: NativePositionState::default(),
            },
        )],
        [],
    )
    .unwrap();
    let before = graph.node(10).cloned();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner;
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    let replay = capture
        .replay_resolved_positions(&mut receiver, |_, _| {
            Ok((10, DestinationContext::Unresolved, Instant::now()))
        })
        .unwrap();
    assert_eq!(replay.positions[0].id, 1);
    assert!(matches!(
        replay.positions[0].outcome,
        NativePositionUpdate::ReparentRequired(_)
    ));
    assert_eq!(graph.node(10).cloned(), before);
}
