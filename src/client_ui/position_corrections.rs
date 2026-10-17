//! Source-profile authoritative corrections, distinct from unacknowledged reports.
use super::*;

pub(super) fn apply(ui: &mut Presentation, before: (u64, Option<crate::world::Transform>)) {
    let after = ui.model.received_player_transform();
    if ui.model.received_transform_revision == before.0 {
        return;
    }
    if let Some(received) = after {
        // A matching last-written report is an echo even when input has already
        // advanced the UI to a newer, not-yet-reported prediction.
        if ui
            .model
            .reported_position
            .zip(ui.model.reported_rotation)
            .is_some_and(|(position, rotation)| received.matches_report(position, rotation))
        {
            return;
        }
        if ui
            .local_position
            .zip(
                ui.local_rotation
                    .or_else(|| before.1.map(|value| value.rotation.map(f64::from))),
            )
            .is_some_and(|(position, rotation)| received.matches_report(position, rotation))
        {
            return;
        }
        let position = received.position.map(f64::from);
        let rotation = received.rotation.map(f64::from);
        ui.local_position = Some(position);
        ui.local_rotation = Some(rotation);
        ui.yaw = placement::received_heading(position, rotation).unwrap_or(ui.yaw);
    } else {
        ui.local_position = None;
        ui.local_rotation = None;
    }
    // Do not send an obsolete dirty prediction back over a received correction.
    // Mouse pitch remains the user's ordinary view control.
    ui.report_dirty = false;
    ui.report_timer = 0.;
}

#[cfg(test)]
mod tests {
    use super::*;
    fn packet(position: [f32; 3], rotation: [f32; 3]) -> Vec<u8> {
        let mut bytes = vec![0x12];
        bytes.extend(42u32.to_be_bytes());
        bytes.extend(0i64.to_be_bytes());
        bytes.push(8);
        for v in position.into_iter().chain(rotation) {
            bytes.extend(v.to_be_bytes());
        }
        bytes.extend([0; 5]);
        bytes
    }
    fn receive(ui: &mut Presentation, bytes: Vec<u8>) {
        let before = (
            ui.model.received_transform_revision,
            ui.model.received_player_transform(),
        );
        ui.model.apply(Event::Packet(bytes)).unwrap();
        apply(ui, before);
    }
    #[test]
    fn authoritative_root_updates_replace_prediction_but_sparse_packets_and_quantized_echoes_do_not()
     {
        let mut ui = Presentation {
            model: ClientModel {
                phase: Phase::Playing,
                avatar: Some(42),
                ..default()
            },
            pitch: -0.4,
            ..default()
        };
        receive(&mut ui, packet([0., 0., 100.], [0.; 3]));
        let predicted = [0.123456789, 0., 101.];
        let rotation = [0., 0., 0.123456789];
        ui.local_position = Some(predicted);
        ui.local_rotation = Some(rotation);
        ui.model
            .apply(Event::SentScene(SceneCommand::PositionReport {
                projected: predicted,
                rotation,
            }))
            .unwrap();
        ui.report_dirty = true;
        receive(
            &mut ui,
            packet(predicted.map(|v| v as f32), rotation.map(|v| v as f32)),
        );
        assert_eq!(ui.local_position, Some(predicted));
        assert_eq!(ui.model.position(), Some(predicted));
        assert!(ui.report_dirty);
        let newer = [predicted[0] + 0.1, 0., 101.];
        ui.local_position = Some(newer);
        receive(
            &mut ui,
            packet(predicted.map(|v| v as f32), rotation.map(|v| v as f32)),
        );
        assert_eq!(
            ui.local_position,
            Some(newer),
            "an echo rolled back newer unsent input"
        );
        assert!(ui.report_dirty);
        ui.local_position = Some(predicted);
        let mut other = packet([1., 2., 3.], [0.; 3]);
        other[1..5].copy_from_slice(&43u32.to_be_bytes());
        receive(&mut ui, other);
        assert_eq!(ui.model.position(), Some(predicted));
        assert_eq!(ui.local_position, Some(predicted));
        let mut sparse = vec![0x12];
        sparse.extend(42u32.to_be_bytes());
        sparse.extend(0i64.to_be_bytes());
        sparse.extend([0; 6]);
        receive(&mut ui, sparse);
        assert_eq!(ui.local_position, Some(predicted));
        receive(&mut ui, packet([0., 0., 99.], [0.; 3]));
        assert_eq!(ui.local_position, Some([0., 0., 99.]));
        assert_eq!(ui.model.position(), ui.local_position);
        assert_eq!(ui.local_rotation, Some([0.; 3]));
        assert!(!ui.report_dirty);
        assert_eq!(ui.report_timer, 0.);
        assert_eq!(ui.pitch, -0.4);
        assert!(ui.model.reported_position.is_none());
        // A same-coordinate reassertion is still authoritative: it must not be
        // mistaken for a sparse omission merely because WorldState is unchanged.
        ui.local_position = Some([0., 0., 102.]);
        ui.model
            .apply(Event::SentScene(SceneCommand::PositionReport {
                projected: [0., 0., 102.],
                rotation: [0.; 3],
            }))
            .unwrap();
        receive(&mut ui, packet([0., 0., 99.], [0.; 3]));
        assert_eq!(ui.local_position, Some([0., 0., 99.]));
        assert_eq!(ui.model.position(), ui.local_position);
        ui.local_rotation = None;
        receive(&mut ui, packet([0., 0., 99.], [0., 0., 0.5]));
        assert_eq!(ui.local_rotation, Some([0., 0., 0.5]));
        assert_ne!(ui.yaw, 0.);
        assert_eq!(ui.pitch, -0.4);
    }
}
