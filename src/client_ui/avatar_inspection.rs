//! Inspection changes the connected camera only, never body coordinates.
use super::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use std::{path::PathBuf, time::Instant};
type InspectionCamera = (With<SliceCamera>, Without<avatar_rendering::LocalAvatar>);

pub(super) fn camera(
    ui: Res<Presentation>,
    state: Res<avatar_rendering::State>,
    roots: Query<&Transform, With<avatar_rendering::LocalAvatar>>,
    mut cameras: Query<(&mut Transform, &mut Projection), InspectionCamera>,
) {
    if !ui.avatar_inspect {
        return;
    }
    let Some((root, center, radius)) = state.focus else {
        return;
    };
    let Ok(body) = roots.get(root) else {
        return;
    };
    let mut target = body.transform_point(center);
    let mut radius = radius * body.scale.max_element();
    if let Some(ground) = ground_point(&ui) {
        // Include actual terrain in the inspection view even when airborne.
        // Camera fitting only: the body remains at its received origin.
        radius += target.distance(ground) * 0.5;
        target = (target + ground) * 0.5;
    }
    let eye = target + body.rotation * Vec3::new(0., 3.5, 1.) * radius;
    for (mut camera, mut projection) in &mut cameras {
        *camera = Transform::from_translation(eye).looking_at(target, body.rotation * Vec3::Z);
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.near = (radius * 0.01).max(1e-7);
        }
    }
}

fn ground_point(ui: &Presentation) -> Option<Vec3> {
    let position = ui.local_position.or_else(|| ui.model.position())?;
    let up = DVec3::from_array(position).normalize_or_zero();
    let radius = super::camera::surface_radius(ui.camera_terrain.as_ref()?, up)?;
    Some((up * radius * ui.scale).as_vec3())
}

/// Explicit native-world-unit reference, not replacement terrain or collision.
pub(super) fn ground_reference(ui: Res<Presentation>, mut gizmos: Gizmos) {
    if !ui.avatar_inspect {
        return;
    }
    let Some(ground) = ground_point(&ui) else {
        return;
    };
    let Some(position) = ui.local_position.or_else(|| ui.model.position()) else {
        return;
    };
    let Some(frame) = placement::tangent_frame(position, ui.yaw) else {
        return;
    };
    let unit = ui.scale as f32;
    let gold = Color::srgb(1., 0.75, 0.1);
    // Small tangent cross at the exact radial hit on the rendered mesh.
    // Lift the diagnostic line 0.02 units to avoid terrain z-fighting.
    let ground = ground + frame.z_axis * unit * 0.02;
    for axis in [frame.x_axis, frame.y_axis] {
        gizmos.line(ground - axis * unit * 2., ground + axis * unit * 2., gold);
    }
    let base = ground + frame.x_axis * unit * 2.;
    gizmos.line(base, base + frame.z_axis * unit * 12., gold);
    for tick in 0..=12 {
        let point = base + frame.z_axis * unit * tick as f32;
        gizmos.line(
            point - frame.x_axis * unit * 0.2,
            point + frame.x_axis * unit * 0.2,
            gold,
        );
    }
}

#[derive(Resource)]
pub(super) struct Capture {
    path: Option<PathBuf>,
    started: Instant,
    frames: u32,
    root: Option<Entity>,
    expected_position: Option<[f64; 3]>,
}
impl Capture {
    pub(super) fn from_environment(auto_login: bool) -> io::Result<Option<Self>> {
        let Some(path) = std::env::var_os("OPENSHORES_TEST_AVATAR_CAPTURE") else {
            return Ok(None);
        };
        let path = PathBuf::from(path);
        if !auto_login || path.symlink_metadata().is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "avatar capture requires synthetic auto-login and a new output path",
            ));
        }
        let expected_position = std::env::var("OPENSHORES_TEST_AVATAR_POSITION")
            .ok()
            .map(|value| -> io::Result<[f64; 3]> {
                let values = value
                    .split(',')
                    .map(str::parse::<f64>)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| io::Error::other("invalid expected avatar position"))?;
                let position: [f64; 3] = values
                    .try_into()
                    .map_err(|_| io::Error::other("expected three avatar coordinates"))?;
                if position.iter().any(|v| !v.is_finite()) {
                    return Err(io::Error::other("expected finite avatar coordinates"));
                }
                Ok(position)
            })
            .transpose()?;
        Ok(Some(Self {
            path: Some(path),
            started: Instant::now(),
            frames: 0,
            root: None,
            expected_position,
        }))
    }
}

pub(super) fn capture(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    state: Res<avatar_rendering::State>,
    mut ui: ResMut<Presentation>,
    mut exit: MessageWriter<AppExit>,
    roots: Query<&Transform, With<avatar_rendering::LocalAvatar>>,
) {
    if capture.started.elapsed().as_secs() > 150 {
        eprintln!(
            "Avatar capture timed out: {} {}",
            ui.avatar_status, ui.avatar_placement_status
        );
        exit.write(AppExit::error());
        return;
    }
    let Some((root, _, _)) = state.focus else {
        capture.root = None;
        capture.frames = 0;
        return;
    };
    if capture.root != Some(root) {
        capture.root = Some(root);
        capture.frames = 0;
    }
    ui.avatar_inspect = true;
    if ui.windows_open || ui.hud {
        ui.windows_open = false;
        ui.hud = false;
        ui.dirty = true;
    }
    capture.frames += 1;
    // Match the offline preview's cold-pipeline warmup, starting AFTER attachment.
    if capture.frames != 600 {
        return;
    }
    if let Some(expected) = capture.expected_position {
        let valid =
            roots
                .get(root)
                .ok()
                .zip(ui.model.position())
                .is_some_and(|(transform, received)| {
                    verify_origin(expected, received, transform.translation, ui.scale)
                });
        if !valid {
            eprintln!(
                "Connected avatar origin mismatch: expected {expected:?}, received {:?}, render {:?}",
                ui.model.position(),
                roots.get(root).ok()
            );
            exit.write(AppExit::error());
            return;
        }
        println!("Connected avatar received/render origin verified: {expected:?}");
    }
    let Some(path) = capture.path.take() else {
        return;
    };
    println!("Connected avatar attached; capturing after 600 warmup frames");
    commands.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            let save = || -> io::Result<()> {
                let image = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(io::Error::other)?;
                let mut output = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?;
                image
                    .to_rgb8()
                    .write_to(
                        &mut output,
                        bevy::image::ImageFormat::Png
                            .as_image_crate_format()
                            .unwrap(),
                    )
                    .map_err(io::Error::other)
            };
            match save() {
                Ok(()) => {
                    println!("Connected avatar capture saved: {}", path.display());
                    exit.write(AppExit::Success);
                }
                Err(error) => {
                    eprintln!("Connected avatar capture failed: {error}");
                    exit.write(AppExit::error());
                }
            }
        },
    );
}

fn verify_origin(expected: [f64; 3], received: [f64; 3], rendered: Vec3, scale: f64) -> bool {
    if !scale.is_finite() || scale <= 0. || !rendered.is_finite() {
        return false;
    }
    expected
        .into_iter()
        .zip(received)
        .zip(rendered.to_array())
        .all(|((expected, received), rendered)| {
            expected.is_finite()
                && received.is_finite()
                && (expected - received).abs() < 0.01
                && (f64::from(rendered) / scale - received).abs() < 0.01
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_gate_rejects_offsets_snapping_double_scale_and_nonfinite_values() {
        let received = [0., 0., 12400.];
        let scale = 3. / 12400.;
        let rendered = Vec3::new(0., 0., 3.);
        assert!(verify_origin(received, received, rendered, scale));
        assert!(!verify_origin([0., 0., 12395.], received, rendered, scale));
        assert!(!verify_origin(
            received,
            received,
            rendered + Vec3::Z * scale as f32,
            scale
        ));
        assert!(!verify_origin(
            received,
            received,
            rendered * scale as f32,
            scale
        ));
        assert!(!verify_origin(received, received, Vec3::NAN, scale));
        assert!(!verify_origin(received, received, rendered, 0.));
    }

    #[test]
    fn inspection_fits_camera_without_moving_or_scaling_body() {
        let mut app = App::new();
        app.insert_resource(Presentation {
            avatar_inspect: true,
            ..default()
        })
        .init_resource::<avatar_rendering::State>()
        .add_systems(Update, camera);
        let body = Transform::from_xyz(0., 0., -3.)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::PI))
            .with_scale(Vec3::splat(0.001));
        let root = app
            .world_mut()
            .spawn((avatar_rendering::LocalAvatar, body))
            .id();
        app.world_mut()
            .resource_mut::<avatar_rendering::State>()
            .focus = Some((root, Vec3::new(0., 0., 0.3), 1.));
        let camera = app
            .world_mut()
            .spawn((
                SliceCamera,
                Transform::default(),
                Projection::Perspective(default()),
            ))
            .id();
        for _ in 0..2 {
            app.update();
            assert_eq!(*app.world().get::<Transform>(root).unwrap(), body);
            let view = app.world().get::<Transform>(camera).unwrap();
            let target = body.transform_point(Vec3::new(0., 0., 0.3));
            assert!((view.translation.distance(target) - 0.001 * 3.5_f32.hypot(1.)).abs() < 1e-6);
            let look = (target - view.translation).normalize();
            assert!((view.rotation * Vec3::NEG_Z - look).length() < 1e-4);
        }
    }
}
