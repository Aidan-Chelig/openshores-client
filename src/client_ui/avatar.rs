//! CPU body preparation only; world-placement/render attachment follows separately.
use super::*;
use crate::client_model::{
    AvatarContentKey, AvatarPreparation, AvatarPreparationStatus, AvatarRequest,
    PreparedNormalAvatar,
};
use std::sync::mpsc::channel;

#[derive(Resource, Default)]
pub(super) struct Preparation(pub Mutex<AvatarPreparation<PreparedNormalAvatar>>);

pub(super) fn preparation(
    mut resource: ResMut<Preparation>,
    setup: Res<assets::Setup>,
    mut ui: ResMut<Presentation>,
) {
    let state = resource.0.get_mut().unwrap();
    let input = (|| -> io::Result<Option<AvatarRequest>> {
        let Some(dna) = ui.model.local_player_dna()? else {
            return Ok(None);
        };
        if !ui.model.world.visible_ids.contains(&dna.entity_id) || setup.prepared.is_none() {
            return Ok(None);
        }
        Ok(Some(AvatarRequest {
            session_generation: ui.model.session_generation,
            entity_id: dna.entity_id,
            content: AvatarContentKey::new(dna.model, setup.revision, 60., true)?,
        }))
    })();
    let mut status = match &input {
        Err(error) => format!("Avatar DNA unavailable: {error}"),
        Ok(None) if ui.model.phase == Phase::Playing && setup.prepared.is_none() => {
            "Avatar assets unavailable: prepare your Hazeron installation.".into()
        }
        Ok(None) if ui.model.phase == Phase::Playing => {
            "Avatar: waiting for the visible selected player's DNA.".into()
        }
        _ => String::new(),
    };
    state.select(input.ok().flatten());
    state.poll();
    if let Some(prepared) = &setup.prepared {
        state.start_next(|request| {
            let mount = prepared.clone();
            let key = request.content.clone();
            let (tx, rx) = channel();
            let failure = tx.clone();
            if let Err(error) = std::thread::Builder::new()
                .name("openshores-avatar".into())
                .spawn(move || {
                    let result = PreparedNormalAvatar::generate(&mount.mount, &key)
                        .map_err(|error| error.to_string());
                    let _ = tx.send(result);
                })
            {
                let _ = failure.send(Err(format!("cannot start avatar worker: {error}")));
            }
            rx
        });
    }
    match state.status() {
        AvatarPreparationStatus::Inactive => {}
        AvatarPreparationStatus::Queued => {
            status = "Avatar: waiting for previous body preparation to finish.".into()
        }
        AvatarPreparationStatus::Running => {
            status = "Preparing received avatar body (normal skin)…".into()
        }
        AvatarPreparationStatus::Ready => {
            status = format!(
                "Avatar body prepared: {} surfaces (normal skin); world placement pending.",
                state.ready().unwrap().1.surfaces.len()
            )
        }
        AvatarPreparationStatus::Failed(error) => {
            status = format!(
                "Avatar body preparation failed: {error}. Check the installation or select supported anatomy; no preview DNA was substituted."
            )
        }
    }
    if ui.avatar_status != status {
        if !status.is_empty() {
            println!("{status}");
        }
        ui.avatar_status = status;
        ui.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_assets_and_native_phase_do_not_start_source_avatar_work() {
        let mut model = ClientModel {
            phase: Phase::Playing,
            avatar: Some(42),
            ..default()
        };
        let mut body = crate::protocol::QtWriter::default();
        for value in [0, 1, 0x24] {
            body.write_u8(value);
        }
        body.write_bytes(Some(&[0; 24])).unwrap();
        for _ in 0..3 {
            body.write_u8(0);
        }
        let mut packet = vec![0x12];
        packet.extend(42u32.to_be_bytes());
        packet.extend(0i64.to_be_bytes());
        packet.push(0);
        packet.extend(body.into_bytes());
        model.world.ingest(&packet).unwrap();
        model.world.visible_ids.insert(42);
        assert!(model.local_player_dna().unwrap().is_some());
        let mut app = App::new();
        app.init_resource::<Preparation>()
            .init_resource::<assets::Setup>()
            .insert_resource(Presentation { model, ..default() })
            .add_systems(Update, preparation);
        app.update();
        assert!(
            app.world()
                .resource::<Presentation>()
                .avatar_status
                .contains("assets unavailable")
        );
        assert_eq!(
            app.world()
                .resource::<Preparation>()
                .0
                .lock()
                .unwrap()
                .status(),
            AvatarPreparationStatus::Inactive
        );
        app.world_mut().resource_mut::<Presentation>().model.phase = Phase::NativePreview;
        app.update();
        assert!(
            app.world()
                .resource::<Presentation>()
                .avatar_status
                .is_empty()
        );
        assert_eq!(
            app.world()
                .resource::<Preparation>()
                .0
                .lock()
                .unwrap()
                .status(),
            AvatarPreparationStatus::Inactive
        );
    }
}
