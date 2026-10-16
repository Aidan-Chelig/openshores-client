//! Native neutral eye point and local-view head suppression; no body offsets.
use super::*;
type CameraFilter = (With<SliceCamera>, Without<avatar_rendering::LocalAvatar>);

pub(super) fn view(
    ui: Res<Presentation>,
    state: Res<avatar_rendering::State>,
    roots: Query<&Transform, With<avatar_rendering::LocalAvatar>>,
    mut cameras: Query<&mut Transform, CameraFilter>,
    mut heads: Query<&mut Visibility, With<avatar_rendering::FirstPersonHead>>,
) {
    // Don't reveal the skull while the rising camera is still inside it.
    let first_person = !ui.avatar_inspect && (!ui.top_down || ui.top_blend < 0.15);
    for mut head in &mut heads {
        *head = if first_person {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    if ui.avatar_inspect {
        return;
    }
    let Some((root, eye)) = state.camera else {
        return;
    };
    let Ok(body) = roots.get(root) else {
        return;
    };
    let Some(position) = ui.local_position.or_else(|| ui.model.position()) else {
        return;
    };
    let Some(frame) = placement::tangent_frame(position, ui.yaw) else {
        return;
    };
    let eye = camera::clear_eye(
        body.transform_point(eye),
        ui.camera_terrain.as_ref(),
        ui.scale,
    );
    for mut camera in &mut cameras {
        *camera = camera::view(eye, frame, ui.pitch, ui.top_height * ui.top_blend, ui.scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_person_uses_native_point_without_moving_body_and_restores_head_for_inspection() {
        let mut app = App::new();
        app.insert_resource(Presentation {
            local_position: Some([0., 0., -3000.]),
            scale: 0.001,
            ..default()
        })
        .init_resource::<avatar_rendering::State>()
        .add_systems(Update, view);
        let body = Transform::from_xyz(0., 0., -3.)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::PI))
            .with_scale(Vec3::splat(0.001));
        let root = app
            .world_mut()
            .spawn((avatar_rendering::LocalAvatar, body))
            .id();
        let point = Vec3::new(0., 0.6, 2.5);
        app.world_mut()
            .resource_mut::<avatar_rendering::State>()
            .camera = Some((root, point));
        let head = app
            .world_mut()
            .spawn((avatar_rendering::FirstPersonHead, Visibility::Inherited))
            .id();
        let torso = app.world_mut().spawn(Visibility::Inherited).id();
        let camera = app
            .world_mut()
            .spawn((SliceCamera, Transform::default()))
            .id();
        app.update();
        assert!(
            (app.world().get::<Transform>(camera).unwrap().translation
                - body.transform_point(point))
            .length()
                < 1e-7
        );
        assert_eq!(app.world().get::<Transform>(root), Some(&body));
        assert_eq!(
            app.world().get::<Visibility>(head),
            Some(&Visibility::Hidden)
        );
        {
            let mut ui = app.world_mut().resource_mut::<Presentation>();
            ui.pitch = -1.2;
            ui.yaw = 0.7;
        }
        app.update();
        let frame = placement::tangent_frame([0., 0., -3000.], 0.7).unwrap();
        let look = frame.y_axis * (-1.2_f32).cos() + frame.z_axis * (-1.2_f32).sin();
        assert!(
            app.world()
                .get::<Transform>(camera)
                .unwrap()
                .forward()
                .distance(look)
                < 1e-6
        );
        assert_eq!(app.world().get::<Transform>(root), Some(&body));
        assert_eq!(
            app.world().get::<Visibility>(torso),
            Some(&Visibility::Inherited)
        );
        let ordinary = *app.world().get::<Transform>(camera).unwrap();
        {
            let mut ui = app.world_mut().resource_mut::<Presentation>();
            ui.top_down = true;
            ui.top_blend = 1.;
            ui.top_height = 8.;
        }
        app.update();
        let elevated = app.world().get::<Transform>(camera).unwrap();
        assert!(elevated.rotation.abs_diff_eq(ordinary.rotation, 1e-6));
        assert!(
            (elevated.translation - ordinary.translation - frame.z_axis * 0.008).length() < 1e-6
        );
        assert_eq!(
            app.world().get::<Visibility>(head),
            Some(&Visibility::Inherited)
        );
        assert_eq!(app.world().get::<Transform>(root), Some(&body));
        app.world_mut()
            .resource_mut::<Presentation>()
            .avatar_inspect = true;
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(head),
            Some(&Visibility::Inherited)
        );
        assert_eq!(app.world().get::<Transform>(root), Some(&body));
    }
}
