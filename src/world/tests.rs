use super::*;
use crate::protocol::write_frame;
use std::collections::BTreeSet;

#[test]
fn commodity_rows_are_not_atoms_and_failed_updates_are_atomic() {
    let mut bytes = vec![0x2b];
    bytes.extend([0; 156]);
    bytes[2] = 31;
    let mut world = WorldState::default();
    world.ingest(&bytes).unwrap();
    assert_eq!(world.commodities.len(), 1);
    assert!(world.entities.is_empty());
    assert!(world.visible_ids.is_empty());
    assert!(world.opaque_packet_counts.is_empty());
    assert_eq!(world.packet_counts[&0x2b], 1);
    assert!(world.ingest(&bytes[..156]).is_err());
    assert_eq!(world.packet_counts[&0x2b], 1);
    assert_eq!(world.commodities.get(31).unwrap().raw, bytes[1..]);
}

fn hex(text: &str) -> Vec<u8> {
    text.trim()
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn geology_updates_are_sparse_and_explicit_empty_clears() {
    let mut bytes = hex(include_str!("../../tests/fixtures/atom-globe.hex"));
    // Existing fixture ends after class terrain: add flag-8 and a mesa.
    let flag_index = bytes.len() - 29;
    bytes[flag_index] |= 8;
    bytes.extend([1, 3]);
    for value in [2.0f32, 1.0, 3.0, 4.0] {
        bytes.extend(value.to_be_bytes());
    }
    let mut world = WorldState::default();
    world.ingest(&bytes).unwrap();
    let geology = world.entities[&7].geology.clone().unwrap();
    assert_eq!(geology.features[0].latitude, 1.0);
    world
        .ingest(&hex(include_str!(
            "../../tests/fixtures/atom-globe-rename.hex"
        )))
        .unwrap();
    assert_eq!(world.entities[&7].geology.as_ref(), Some(&geology));
    bytes.truncate(bytes.len() - 18);
    bytes.push(0);
    world.ingest(&bytes).unwrap();
    assert!(
        world.entities[&7]
            .geology
            .as_ref()
            .unwrap()
            .features
            .is_empty()
    );
}

#[test]
fn globe_rename_preserves_surface_and_rejects_truncated_parameters() {
    let bytes = hex(include_str!("../../tests/fixtures/atom-globe.hex"));
    let mut world = WorldState::default();
    world.ingest(&bytes).unwrap();
    let surface = world.entities[&7].globe_surface.unwrap();
    assert_eq!(
        (
            surface.size,
            surface.atmosphere_type,
            surface.atmosphere_density,
            surface.water
        ),
        (8, 0, 60, 40)
    );
    assert_eq!(
        world.entities[&7].world_identity.as_ref().unwrap().orbit_au,
        1.0
    );
    assert!((surface.terrain.altitude_msl(8, 0.0, 0.0).unwrap() - 216.0207753854408).abs() < 1e-8);
    world
        .ingest(&hex(include_str!(
            "../../tests/fixtures/atom-globe-rename.hex"
        )))
        .unwrap();
    assert_eq!(world.entities[&7].name.as_deref(), Some("Renamed"));
    assert_eq!(world.entities[&7].globe_surface, Some(surface));
    for end in 0..bytes.len() {
        assert!(decode_atom(&bytes[..end]).is_err());
    }
}

#[test]
fn python_sector_fixture_and_sparse_rename() {
    let full = hex(include_str!("../../tests/fixtures/atom-sector.hex"));
    let mut world = WorldState::default();
    world.ingest(&full).unwrap();
    let original = world.entities[&42].clone();
    assert_eq!(original.parent, Some(117));
    assert_eq!(original.name.as_deref(), Some("Sector 🚀"));
    assert_eq!(original.transform.unwrap().position, [1.25, -2.5, 3.0]);
    world
        .ingest(&hex(include_str!(
            "../../tests/fixtures/atom-sector-rename.hex"
        )))
        .unwrap();
    assert_eq!(world.entities[&42].name.as_deref(), Some("Renamed"));
    assert_eq!(world.entities[&42].transform, original.transform);
    assert_eq!(world.entities[&42].parent, original.parent);
    assert_eq!(world.entities[&42].timestamp_ms, 2000);
    for end in 0..full.len() {
        assert!(decode_atom(&full[..end]).is_err());
    }
}

#[test]
fn cycles_and_type_conflicts_leave_state_unchanged() {
    let mut world = WorldState::default();
    let base = decode_atom(&hex(include_str!("../../tests/fixtures/atom-sector.hex")))
        .unwrap()
        .unwrap();
    world.apply(base.clone()).unwrap();
    let original = world.entities.clone();
    let mut cycle = base.clone();
    cycle.parent = Some(42);
    assert!(world.apply(cycle).is_err());
    let mut parent = base.clone();
    parent.id = 117;
    parent.parent = Some(42);
    assert!(world.apply(parent).is_err());
    let mut conflict = base;
    conflict.kind = AtomKind::Person;
    assert!(world.apply(conflict).is_err());
    assert_eq!(world.entities, original);
}

#[test]
fn replay_tracks_manifest_visibility_and_rejects_truncation() {
    let atom = hex(include_str!("../../tests/fixtures/atom-sector.hex"));
    let mut capture = Vec::new();
    write_frame(&mut capture, &atom).unwrap();
    write_frame(&mut capture, &hex("18000000010100020000002a00000063")).unwrap();
    write_frame(&mut capture, &[0xfe, 1, 2]).unwrap();
    let mut world = read_capture(&mut capture.as_slice()).unwrap();
    assert_eq!(world.missing_manifest_ids(), [99]);
    assert_eq!(world.unresolved_parents(), BTreeSet::from([117]));
    assert_eq!(world.opaque_packet_counts[&0xfe], 1);
    world.ingest(&hex("1800000002010000")).unwrap();
    assert!(world.visible_ids.is_empty());
    assert_eq!(world.entities.len(), 1);
    assert!(read_capture(&mut [0xc0].as_slice()).is_err());
    assert!(read_capture(&mut [2, 1].as_slice()).is_err());
}
