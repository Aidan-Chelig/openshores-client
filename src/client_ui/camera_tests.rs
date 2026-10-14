use super::*;

#[test]
fn underground_eye_is_clamped_to_rendered_terrain_without_moving_avatar() {
    let terrain = TerrainMesh::generate(crate::terrain_mesh::demonstration_surface(), 32).unwrap();
    let scale = 3.0 / terrain.statistics().base_radius;
    for direction in [
        DVec3::Z,
        -DVec3::Z,
        DVec3::X,
        DVec3::Y,
        DVec3::new(1.0, 2.0, 3.0).normalize(),
    ] {
        let floor = surface_radius(&terrain, direction).unwrap();
        let position = (direction * (floor - 100.0)).to_array();
        let view = eye(position, Some(&terrain), scale).as_dvec3();
        assert!(view.length() / scale > floor + 1.69);
        assert!((view.normalize() - direction).length() < 1e-6);
        assert_eq!(position, (direction * (floor - 100.0)).to_array());
    }
}

#[test]
fn airborne_eye_and_near_plane_use_world_units_at_each_globe_scale() {
    let terrain = TerrainMesh::generate(crate::terrain_mesh::demonstration_surface(), 32).unwrap();
    let floor = surface_radius(&terrain, DVec3::Z).unwrap();
    for scale in [0.001, 0.0001, 0.00001] {
        let position = [0.0, 0.0, floor + 100.0];
        let view = eye(position, Some(&terrain), scale);
        assert!((f64::from(view.z) / scale - floor - 101.7).abs() < 0.01);
        assert!((f64::from(near(scale)) / scale - NEAR_DISTANCE).abs() < 1e-6);
        assert!(f64::from(near(scale)) < EYE_HEIGHT * scale);
    }
}
