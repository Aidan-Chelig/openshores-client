//! Movement-driven native poses on persistent meshes. No installation I/O here.
use super::*;
use crate::world::native_body::NativeDpGait;

#[derive(Resource, Default)]
pub(super) struct State {
    owner: Option<(u64, u32, Entity)>,
    elapsed: f64,
    standing: bool,
    failure: Option<String>,
}
fn status(ui: &mut Presentation, text: &str) {
    if ui.avatar_animation_status != text {
        ui.avatar_animation_status = text.into();
        ui.dirty = true;
        if !text.is_empty() {
            println!("{text}");
        }
    }
}
pub(super) fn animate(
    mut state: ResMut<State>,
    mut ui: ResMut<Presentation>,
    prepared: Res<avatar::Preparation>,
    mut rendered: ResMut<avatar_rendering::State>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    let ready = {
        let prep = prepared.0.lock().unwrap();
        prep.ready()
            .map(|(request, body)| (request.clone(), body.clone()))
    };
    let ready = ready.filter(|(request, _)| {
        rendered
            .appearance()
            .is_some_and(|(owner, _)| owner == request)
    });
    let Some(((request, body), (root, _))) = ready.zip(rendered.camera) else {
        *state = State::default();
        status(&mut ui, "");
        return;
    };
    let Some(animation) = &body.animation else {
        return;
    };
    let owner = (request.session_generation, request.entity_id, root);
    if state.owner != Some(owner) {
        *state = State {
            owner: Some(owner),
            ..default()
        };
    }
    if let Some(error) = &state.failure {
        status(&mut ui, error);
        return;
    }
    // This is local movement prediction, not an interpretation of the source
    // server's opaque person_pose byte or a native production locomotion solver.
    let speed = ui.avatar_motion_speed;
    let gait = if speed >= 8. {
        Some(NativeDpGait::Run)
    } else if speed > 0. {
        Some(NativeDpGait::Walk)
    } else {
        None
    };
    if gait.is_none() && state.standing {
        return;
    }
    state.elapsed += time.delta_secs_f64();
    let result = (|| -> io::Result<()> {
        let frame =
            animation
                .lock()
                .unwrap()
                .sample(gait, speed, (state.elapsed * 1000.) as i64)?;
        let ids = rendered.mesh_ids();
        if ids.len() != body.bindings.len() {
            return Err(io::Error::other("animation mesh binding count changed"));
        }
        for (id, bindings) in ids.into_iter().zip(&body.bindings) {
            let mut mesh = meshes
                .get_mut(id)
                .ok_or_else(|| io::Error::other("animation mesh disappeared"))?;
            let mut positions = Vec::with_capacity(bindings.len());
            let mut normals = Vec::with_capacity(bindings.len());
            let mut colors = Vec::with_capacity(bindings.len());
            for &(surface, corner) in bindings {
                let corner = &frame.surfaces[surface].corners[corner];
                let [x, y, z, w] = corner.position;
                if w <= 0. || ![x, y, z, w].iter().all(|v| v.is_finite()) {
                    return Err(io::Error::other("invalid animated homogeneous vertex"));
                }
                positions.push([x / w, y / w, z / w]);
                normals.push(corner.normal);
                colors.push([w, 1., 1., corner.top_face]);
            }
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
        rendered.camera = Some((root, Vec3::from_array(frame.eye.map(|v| v as f32))));
        Ok(())
    })();
    if let Err(error) = result {
        let message = format!("Avatar animation failed: {error}");
        status(&mut ui, &message);
        state.failure = Some(message);
    } else {
        state.standing = gait.is_none();
        status(
            &mut ui,
            match gait {
                None => "Avatar animation: standing",
                Some(NativeDpGait::Walk) => "Avatar animation: walking",
                Some(NativeDpGait::Run) => "Avatar animation: running",
            },
        );
    }
}
