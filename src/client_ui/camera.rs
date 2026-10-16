//! View-only terrain clearance. Does not change avatar state or position reports.
use super::*;

// Prototype world-unit eye height, not a reconstructed creature/body dimension.
const EYE_HEIGHT: f64 = 1.7;
const NEAR_DISTANCE: f64 = 0.05;

pub(super) fn transition(mut ui: ResMut<Presentation>, time: Res<Time>) {
    let target = if ui.top_down { 1. } else { 0. };
    // Frame-rate-independent quick lerp; reversing mid-transition starts here.
    let amount = 1. - (-20. * time.delta_secs()).exp();
    ui.top_blend += (target - ui.top_blend) * amount;
    if (target - ui.top_blend).abs() < 0.001 {
        ui.top_blend = target;
    }
}

/// Elevated view translates the ordinary eye only; mouse-look is identical.
pub(super) fn view(eye: Vec3, frame: Mat3, pitch: f32, height: f32, scale: f64) -> Transform {
    let eye = eye + frame.z_axis * height * scale as f32;
    let look = frame.y_axis * pitch.cos() + frame.z_axis * pitch.sin();
    Transform::from_translation(eye).looking_to(look, frame.z_axis)
}

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

/// Preserve the prepared native eye, only lifting a buried CAMERA above the
/// actual diagnostic mesh. Never feeds back into entity/body state.
pub(super) fn clear_eye(eye: Vec3, terrain: Option<&TerrainMesh>, scale: f64) -> Vec3 {
    let point = eye.as_dvec3() / scale;
    let up = point.normalize_or_zero();
    let Some(floor) = terrain.and_then(|mesh| surface_radius(mesh, up)) else {
        return eye;
    };
    if point.length() >= floor + NEAR_DISTANCE {
        return eye;
    }
    (up * (floor + NEAR_DISTANCE) * scale).as_vec3()
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
