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
    let target = body.transform_point(center);
    let radius = radius * body.scale.max_element();
    let eye = target + body.rotation * Vec3::new(0., 3.5, 1.) * radius;
    for (mut camera, mut projection) in &mut cameras {
        *camera = Transform::from_translation(eye).looking_at(target, body.rotation * Vec3::Z);
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.near = (radius * 0.01).max(1e-7);
        }
    }
}

#[derive(Resource)]
pub(super) struct Capture {
    path: Option<PathBuf>,
    started: Instant,
    frames: u32,
    root: Option<Entity>,
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
        Ok(Some(Self {
            path: Some(path),
            started: Instant::now(),
            frames: 0,
            root: None,
        }))
    }
}

pub(super) fn capture(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    state: Res<avatar_rendering::State>,
    mut ui: ResMut<Presentation>,
    mut exit: MessageWriter<AppExit>,
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

#[cfg(test)]
mod tests {
    use super::*;

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
