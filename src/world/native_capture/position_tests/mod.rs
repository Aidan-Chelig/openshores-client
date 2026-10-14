use super::*;
mod aliases;
mod failures;
use crate::{
    atom_position::NativePositionState,
    protocol::write_frame,
    world::{
        native_atom::NativeAtomKind,
        native_runaways::NativeRunaways,
        native_subatoms::{NativeCacheNode, NativeChildList, NativeNormalSubatoms},
        native_tick_tock::store::{RegistrationStore, StoreContext},
    },
};

struct Owner;
impl ReparentRegistrationContext for Owner {
    type Context<'a> = NeverContext;
    fn context<'a>(&'a mut self, _: &'a mut NativeNormalSubatoms, _: u32) -> NeverContext {
        panic!("no TickTock in this fixture")
    }
}
struct NeverContext;
impl StoreContext for NeverContext {
    fn must_not_tick_tock(&mut self, _: &RegistrationStore) -> bool {
        unreachable!()
    }
    fn is_ticker(&mut self, _: &RegistrationStore) -> bool {
        unreachable!()
    }
    fn is_tocker(&mut self, _: &RegistrationStore) -> u32 {
        unreachable!()
    }
    fn find_parent_solar_system(
        &mut self,
        _: &RegistrationStore,
    ) -> Option<crate::world::native_tick_tock::SolarSystem> {
        unreachable!()
    }
    fn is_ticking_contents(
        &mut self,
        _: &RegistrationStore,
        _: crate::world::native_tick_tock::SolarSystem,
    ) -> bool {
        unreachable!()
    }
    fn solar_field_3a9(
        &mut self,
        _: &RegistrationStore,
        _: crate::world::native_tick_tock::SolarSystem,
    ) -> u8 {
        unreachable!()
    }
    fn atom_type(&mut self, _: &RegistrationStore) -> u8 {
        unreachable!()
    }
}
fn graph() -> NativeNormalSubatoms {
    NativeNormalSubatoms::new(
        [(
            1,
            NativeCacheNode {
                kind: NativeAtomKind::Item,
                field_1c: 0,
                position: NativePositionState::default(),
            },
        )],
        [(
            0,
            NativeChildList {
                available: true,
                children: vec![1],
            },
        )],
    )
    .unwrap()
}
fn packet(id: u32, parent: u32, x: f64, timestamp: i64, position: bool) -> Vec<u8> {
    let mut bytes = vec![0x16];
    bytes.extend(id.to_be_bytes());
    bytes.extend(timestamp.to_be_bytes());
    bytes.push(u8::from(position));
    if position {
        bytes.push(0x8f);
        bytes.extend(parent.to_be_bytes());
        for value in [x, 2., 3.] {
            bytes.extend(value.to_be_bytes());
        }
    }
    bytes.extend([0xde, 0xad]);
    bytes
}
fn capture(packets: &[Vec<u8>]) -> NativeCapture {
    let mut bytes = Vec::new();
    for packet in packets {
        write_frame(&mut bytes, packet).unwrap();
    }
    NativeCapture::read(&mut bytes.as_slice(), CaptureLimits::default()).unwrap()
}

#[test]
fn replay_preserves_capture_order_raw_bytes_and_unresolved_obligations_without_timestamp_filtering()
{
    let capture = capture(&[
        vec![0xff],
        packet(1, 0, 12345678.125, 100, true),
        packet(99, 0, 0., 1, false),
        packet(1, 0, 3., 10, true),
        packet(1, 7, 4., 5, true),
    ]);
    let originals = capture
        .frames
        .iter()
        .map(|frame| frame.payload().to_vec())
        .collect::<Vec<_>>();
    let mut graph = graph();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner;
    let now = Instant::now();
    let mut seen = Vec::new();
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    let replay = capture
        .replay_positions(&mut receiver, |index, prefix| {
            seen.push((index, prefix.base.timestamp_ms));
            (
                DestinationContext::Unresolved,
                now + std::time::Duration::from_millis(index as u64),
            )
        })
        .unwrap();
    assert_eq!(seen, [(1, 100), (3, 10), (4, 5)]);
    assert_eq!(replay.non_atom_frames, 1);
    assert_eq!(replay.no_position_frames, 1);
    assert_eq!(replay.positions.len(), 3);
    assert!(matches!(
        replay.positions[2].outcome,
        NativePositionUpdate::ReparentRequired(_)
    ));
    assert_eq!(graph.node(1).unwrap().position.position.local[3][0], 3.);
    assert_eq!(
        graph
            .node(1)
            .unwrap()
            .position
            .position_update_age_at(now + std::time::Duration::from_millis(3)),
        Some(std::time::Duration::ZERO)
    );
    assert!(queue.is_empty());
    assert_eq!(
        capture
            .frames
            .iter()
            .map(|frame| frame.payload().to_vec())
            .collect::<Vec<_>>(),
        originals
    );
}

#[test]
fn failed_frame_returns_prior_results_and_state_without_evaluating_later_context() {
    let capture = capture(&[
        packet(1, 0, 12.5, 1, true),
        packet(2, 0, 99., 2, true),
        packet(1, 0, 3., 3, true),
    ]);
    let mut graph = graph();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner;
    let now = Instant::now();
    let mut calls = 0;
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    let error = capture
        .replay_positions(&mut receiver, |_, _| {
            calls += 1;
            (DestinationContext::Unresolved, now)
        })
        .unwrap_err();
    assert_eq!(error.frame, Some(1));
    assert_eq!(error.completed.positions.len(), 1);
    assert_eq!(calls, 2);
    assert_eq!(graph.node(1).unwrap().position.position.local[3][0], 12.5);
    assert!(error.to_string().contains("frame 1"));
}

#[test]
fn public_capture_frame_budget_is_rechecked_before_any_owner_effects() {
    let mut capture = capture(&[packet(1, 0, 12.5, 1, true)]);
    capture.frames.resize(
        CaptureLimits::default().frames + 1,
        capture.frames[0].clone(),
    );
    let mut graph = graph();
    let before = graph.node(1).cloned();
    let mut queue = NativeRunaways::default();
    let mut store = RegistrationStore::default();
    let mut owner = Owner;
    let mut receiver = NativePositionReceiver {
        graph: &mut graph,
        runaways: &mut queue,
        registrations: &mut store,
        owner: &mut owner,
    };
    let error = capture
        .replay_positions(&mut receiver, |_, _| panic!("budget checked first"))
        .unwrap_err();
    assert_eq!(error.frame, None);
    assert!(error.completed.positions.is_empty());
    assert_eq!(graph.node(1).cloned(), before);
}
