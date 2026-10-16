//! View-only terrain clearance. Does not change avatar state or position reports.
use super::*;

// Prototype world-unit eye height, not a reconstructed creature/body dimension.
const EYE_HEIGHT: f64 = 1.7;
const NEAR_DISTANCE: f64 = 0.05;

/// Intersect a radial ray with the actual tessellation, not the analytic height:
/// the coarse diagnostic mesh can sit above or below that continuous surface.
pub(super) fn surface_radius(terrain: &TerrainMesh, up: DVec3) -> Option<f64> {
    terrain
        .triangles()
        .iter()
        .filter_map(|triangle| {
            let [a, b, c] =
                triangle.map(|index| DVec3::from_array(terrain.positions()[index as usize]));
            let e1 = b - a;
            let e2 = c - a;
            let p = up.cross(e2);
            let determinant = e1.dot(p);
            if determinant.abs() <= f64::EPSILON {
                return None;
            }
            let inverse = determinant.recip();
            let t = -a;
            let u = t.dot(p) * inverse;
            let q = t.cross(e1);
            let v = up.dot(q) * inverse;
            let radius = e2.dot(q) * inverse;
            let tolerance = 1e-9;
            (u >= -tolerance && v >= -tolerance && u + v <= 1.0 + tolerance && radius > 0.0)
                .then_some(radius)
        })
        .reduce(f64::max)
}

pub(super) fn eye(position: [f64; 3], terrain: Option<&TerrainMesh>, scale: f64) -> Vec3 {
    let point = DVec3::from_array(position);
    let radial = point.normalize_or_zero();
    let up = if radial.length_squared() > 0.0 {
        radial
    } else {
        DVec3::Z
    };
    let floor = terrain
        .and_then(|mesh| surface_radius(mesh, up))
        .unwrap_or(point.length());
    (up * (point.length().max(floor) + EYE_HEIGHT) * scale).as_vec3()
}

pub(super) fn near(scale: f64) -> f32 {
    (NEAR_DISTANCE * scale).max(1e-7) as f32
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
