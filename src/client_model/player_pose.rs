//! Explicit source-profile pose adapter. DNA prefix tags are never pose channels.
use super::{ClientModel, Phase};
use crate::{
    person::PersonState,
    world::{AtomKind, native_creature::NativeDpBodyPoseState},
};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalPlayerGesture {
    Wave,
    Point,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalPlayerPose {
    pub received: [u8; 10],
    pub native: NativeDpBodyPoseState,
    pub speed: f32,
    /// Source-profile radians, bounded to native body30; body34 is zero.
    /// Rendering input only, not Creature::TiltHeadTo runtime side effects.
    pub head_angles: [f32; 2],
    pub predicted: bool,
}
impl LocalPlayerPose {
    /// Held local posture over source Stand only; received actions stay intact.
    pub fn with_crouch(mut self, crouched: bool) -> Self {
        if crouched && self.received == [0x24; 10] {
            self.native.fields_3c = [if self.speed > 0. { 0x11 } else { 0x10 }; 10];
            self.predicted = true;
        }
        self
    }
    /// Local source-profile prediction is permitted only over received Stand.
    /// Keep the predicted gait on every other channel and never mutate receipt.
    /// Apply to a freshly selected base pose; None leaves that pose unchanged.
    pub fn with_gesture(mut self, gesture: Option<LocalPlayerGesture>) -> Self {
        if self.received == [0x24; 10]
            && let Some(gesture) = gesture
        {
            self.native.fields_3c[2] = match gesture {
                LocalPlayerGesture::Wave => 0x30,
                LocalPlayerGesture::Point => 0x1f,
            };
            self.predicted = true;
        }
        self
    }
    /// Explicit inverse table for this source profile's local predictions only.
    /// Never serialize arbitrary native pose words as source enum bytes.
    pub fn source_report_channels(&self) -> Option<[u8; 10]> {
        if self.received != [0x24; 10] || self.native.field_6c {
            return None;
        }
        let mut result = [0; 10];
        for (index, (&native, byte)) in self.native.fields_3c.iter().zip(&mut result).enumerate() {
            *byte = match native {
                0x10 => 0x10,
                0x11 => 0x11,
                0x24 => 0x24,
                0x25 => 0x25,
                0x26 => 0x26,
                0x30 if index == 2 => 0x30,
                0x1f if index == 2 => 0x1f,
                _ => return None,
            };
        }
        Some(result)
    }
    pub fn is_standing(&self) -> bool {
        self.native.fields_3c == [0x24; 10]
    }
}
impl ClientModel {
    pub fn local_player_pose(&self, predicted_speed: f32) -> io::Result<Option<LocalPlayerPose>> {
        if self.phase != Phase::Playing {
            return Ok(None);
        }
        let Some(entity) = self.avatar.and_then(|id| self.world.entities.get(&id)) else {
            return Ok(None);
        };
        if entity.kind != AtomKind::Person {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "selected pose owner is not a person",
            ));
        }
        entity
            .person
            .as_ref()
            .map_or(Ok(None), |person| select(person, predicted_speed))
    }
}
fn select(person: &PersonState, predicted_speed: f32) -> io::Result<Option<LocalPlayerPose>> {
    let Some(received) = person.poses else {
        return Ok(None);
    };
    if !predicted_speed.is_finite() || predicted_speed < 0. {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid predicted animation speed",
        ));
    }
    if received.iter().enumerate().any(|(index, code)| {
        !(matches!(code, 0x10..=0x15 | 0x18..=0x19 | 0x1b..=0x1c | 0x20 | 0x24..=0x26 | 0x35)
            || matches!(index, 2 | 5) && matches!(code, 0x1f | 0x30))
    }) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("unsupported received action channels {received:02x?}"),
        ));
    }
    if person.pose_parameter != Some(0) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "received pose parameter requires its own supported adapter",
        ));
    }
    let tilt = person.head_tilt.unwrap_or(0.);
    if !tilt.is_finite() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "nonfinite received head tilt",
        ));
    }
    // The supplied source writer sends float radians, not the outgoing i16
    // delta-tick angle. All admitted general/part channels exclude native
    // TiltHeadTo's special 0x39/0x0f/0x17 gates. This maps stored rendering
    // angles only: native Creature timers, parent gates and callbacks are not
    // claimed by the source-profile presentation path.
    let head_angles = [
        tilt.clamp(f32::from_bits(0xbfc9_0fdb), f32::from_bits(0x3fc9_0fdb)),
        0.,
    ];
    // This narrow table is explicit source->native compatibility, not packet
    // auto-detection. Predict movement only over the source's neutral Stand.
    let predicted = received == [0x24; 10] && predicted_speed > 0.;
    let fields_3c = if predicted {
        [if predicted_speed >= 8. { 0x25 } else { 0x26 }; 10]
    } else {
        received.map(u32::from)
    };
    // Received pose-only locomotion lacks a velocity field in this profile:
    // nominal rate 1 is a declared rendering policy, never invented world motion.
    let speed = if predicted_speed > 0. {
        predicted_speed
    } else if received != [0x24; 10] {
        1.
    } else {
        0.
    };
    Ok(Some(LocalPlayerPose {
        received,
        native: NativeDpBodyPoseState {
            fields_3c,
            field_6c: false,
        },
        speed,
        head_angles,
        predicted,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crouch_prediction_preserves_received_authority_and_composes_arm_gestures() {
        let mut person = PersonState {
            poses: Some([0x24; 10]),
            pose_parameter: Some(0),
            head_tilt: Some(0.25),
            ..Default::default()
        };
        for speed in [0., 4.] {
            let base = select(&person, speed).unwrap().unwrap();
            let code = if speed > 0. { 0x11 } else { 0x10 };
            let crouch = base.clone().with_crouch(true);
            assert_eq!(crouch.native.fields_3c, [code; 10]);
            assert_eq!(crouch.received, [0x24; 10]);
            assert_eq!(crouch.head_angles, [0.25, 0.]);
            assert_eq!(crouch.source_report_channels(), Some([code as u8; 10]));
            let wave = crouch.with_gesture(Some(LocalPlayerGesture::Wave));
            let mut expected = [code as u8; 10];
            expected[2] = 0x30;
            assert_eq!(wave.source_report_channels(), Some(expected));
            assert_eq!(base.clone().with_crouch(false), base);
        }
        for action in [0x10, 0x11, 0x12, 0x15, 0x25] {
            person.poses = Some([action; 10]);
            let received = select(&person, 4.).unwrap().unwrap();
            assert_eq!(received.clone().with_crouch(true), received);
            assert!(received.source_report_channels().is_none());
        }
    }
    #[test]
    fn local_arm_gestures_preserve_gait_receipt_and_head_angles_but_yield_to_server_actions() {
        let mut person = PersonState {
            poses: Some([0x24; 10]),
            pose_parameter: Some(0),
            head_tilt: Some(0.25),
            ..Default::default()
        };
        for (speed, gait) in [(0., 0x24), (4., 0x26), (12., 0x25)] {
            let base = select(&person, speed).unwrap().unwrap();
            assert_eq!(base.clone().with_gesture(None), base);
            for (gesture, code) in [
                (LocalPlayerGesture::Wave, 0x30),
                (LocalPlayerGesture::Point, 0x1f),
            ] {
                let selected = base.clone().with_gesture(Some(gesture));
                let mut expected = [gait; 10];
                expected[2] = code;
                assert_eq!(selected.native.fields_3c, expected);
                assert_eq!(
                    selected.source_report_channels(),
                    Some(expected.map(|v| v as u8))
                );
                assert_eq!(selected.received, [0x24; 10]);
                assert_eq!(selected.head_angles, [0.25, 0.]);
                assert_eq!(selected.speed, base.speed);
                assert!(selected.predicted);
            }
        }
        for code in [0x10, 0x11, 0x12, 0x15, 0x1c, 0x25] {
            person.poses = Some([code; 10]);
            let base = select(&person, 12.).unwrap().unwrap();
            assert!(base.source_report_channels().is_none());
            assert_eq!(
                base.clone().with_gesture(Some(LocalPlayerGesture::Wave)),
                base
            );
        }
        person.poses = Some([0x24; 10]);
        let mut invalid = select(&person, 0.).unwrap().unwrap();
        for code in [0x13, 0x100, 0x1f, 0x30] {
            invalid.native.fields_3c[0] = code;
            assert!(invalid.source_report_channels().is_none());
        }
    }
    #[test]
    fn sparse_source_head_tilt_updates_preserve_channels_and_reset_to_zero() {
        let mut model = ClientModel {
            phase: Phase::Playing,
            avatar: Some(42),
            ..Default::default()
        };
        for (tilt, with_pose, expected) in [
            (Some(0.25f32), true, 0.25),
            (None, false, 0.25),
            (Some(-0.5), false, -0.5),
            (Some(0.), false, 0.),
        ] {
            let mut body = crate::protocol::QtWriter::default();
            body.write_u8(0);
            body.write_u8(if tilt.is_some() { 2 } else { 0 } | if with_pose { 8 } else { 0 });
            if let Some(tilt) = tilt {
                body.write_u32(tilt.to_bits());
            }
            if with_pose {
                for _ in 0..10 {
                    body.write_u8(0x24);
                }
                body.write_u32(0);
            }
            for _ in 0..3 {
                body.write_u8(0);
            }
            let mut packet = vec![0x12];
            packet.extend(42u32.to_be_bytes());
            packet.extend(0i64.to_be_bytes());
            packet.push(0);
            packet.extend(body.into_bytes());
            model.world.ingest(&packet).unwrap();
            let selected = model.local_player_pose(0.).unwrap().unwrap();
            assert!(selected.is_standing());
            assert_eq!(selected.head_angles, [expected, 0.]);
            assert_eq!(selected.received, [0x24; 10]);
        }
    }
    #[test]
    fn received_pose_updates_use_selected_source_owner_and_preserve_omissions() {
        fn packet(poses: Option<[u8; 10]>) -> Vec<u8> {
            let mut body = crate::protocol::QtWriter::default();
            body.write_u8(0);
            body.write_u8(if poses.is_some() { 8 } else { 0 });
            if let Some(poses) = poses {
                for code in poses {
                    body.write_u8(code);
                }
                body.write_u32(0);
            }
            for _ in 0..3 {
                body.write_u8(0);
            }
            let mut result = vec![0x12];
            result.extend(42u32.to_be_bytes());
            result.extend(0i64.to_be_bytes());
            result.push(0);
            result.extend(body.into_bytes());
            result
        }
        let mut model = ClientModel {
            phase: Phase::Playing,
            avatar: Some(42),
            ..Default::default()
        };
        assert!(model.local_player_pose(0.).unwrap().is_none());
        model.world.ingest(&packet(Some([0x24; 10]))).unwrap();
        assert!(model.local_player_pose(0.).unwrap().unwrap().is_standing());
        let mut mixed = [0x24; 10];
        mixed[2] = 0x1f;
        model.world.ingest(&packet(Some(mixed))).unwrap();
        assert_eq!(
            model
                .local_player_pose(12.)
                .unwrap()
                .unwrap()
                .native
                .fields_3c,
            mixed.map(u32::from)
        );
        mixed[2] = 0x25;
        model.world.ingest(&packet(Some(mixed))).unwrap();
        model.world.ingest(&packet(None)).unwrap();
        assert_eq!(
            model.local_player_pose(4.).unwrap().unwrap().received,
            mixed
        );
        model.world.ingest(&packet(Some([0x17; 10]))).unwrap();
        assert!(model.local_player_pose(12.).is_err());
        for code in [0x10, 0x11, 0x13, 0x14, 0x18, 0x19, 0x1b, 0x20, 0x35] {
            model.world.ingest(&packet(Some([code; 10]))).unwrap();
            assert_eq!(
                model
                    .local_player_pose(12.)
                    .unwrap()
                    .unwrap()
                    .native
                    .fields_3c,
                [u32::from(code); 10]
            );
        }
        model.world.ingest(&packet(Some([0x1c; 10]))).unwrap();
        assert_eq!(
            model
                .local_player_pose(0.)
                .unwrap()
                .unwrap()
                .native
                .fields_3c,
            [0x1c; 10]
        );
        model.world.ingest(&packet(Some([0x25; 10]))).unwrap();
        assert_eq!(
            model
                .local_player_pose(0.)
                .unwrap()
                .unwrap()
                .native
                .fields_3c,
            [0x25; 10]
        );
        model.phase = Phase::NativePreview;
        assert!(model.local_player_pose(12.).unwrap().is_none());
        model.phase = Phase::Playing;
        model.avatar = Some(43);
        assert!(model.local_player_pose(12.).unwrap().is_none());
    }
    #[test]
    fn source_channels_are_preserved_and_dna_tag_is_never_a_pose_fallback() {
        let mut person = PersonState {
            dna_tag: Some(0x25),
            ..Default::default()
        };
        assert!(select(&person, 4.).unwrap().is_none());
        person.poses = Some([0x24; 10]);
        person.pose_parameter = Some(0);
        let walk = select(&person, 4.).unwrap().unwrap();
        assert!(walk.predicted);
        assert_eq!(walk.received, [0x24; 10]);
        assert_eq!(walk.native.fields_3c, [0x26; 10]);
        assert_eq!(
            select(&person, 12.).unwrap().unwrap().native.fields_3c,
            [0x25; 10]
        );
        let mut mixed = [0x24; 10];
        mixed[2] = 0x25;
        mixed[4] = 0x26;
        person.poses = Some(mixed);
        let selected = select(&person, 12.).unwrap().unwrap();
        assert!(!selected.predicted);
        assert_eq!(selected.native.fields_3c, mixed.map(u32::from));
        assert_eq!(select(&person, 0.).unwrap().unwrap().speed, 1.);
        person.poses = Some([0x12; 10]);
        assert_eq!(
            select(&person, 12.).unwrap().unwrap().native.fields_3c,
            [0x12; 10]
        );
        person.poses = Some([0x15; 10]);
        assert_eq!(
            select(&person, 0.).unwrap().unwrap().native.fields_3c,
            [0x15; 10]
        );
        for code in [0x13, 0x14, 0x20] {
            person.poses = Some([code; 10]);
            let selected = select(&person, 12.).unwrap().unwrap();
            assert!(!selected.predicted);
            assert_eq!(selected.native.fields_3c, [u32::from(code); 10]);
            assert!(selected.source_report_channels().is_none());
            assert_eq!(
                selected
                    .clone()
                    .with_crouch(true)
                    .with_gesture(Some(LocalPlayerGesture::Wave)),
                selected
            );
        }
        person.poses = Some([0x17; 10]);
        assert!(select(&person, 12.).is_err());
        person.poses = Some([0x1c; 10]);
        let jump = select(&person, 12.).unwrap().unwrap();
        assert!(!jump.predicted);
        assert_eq!(jump.native.fields_3c, [0x1c; 10]);
        for code in [0x18, 0x19, 0x1b, 0x35] {
            person.poses = Some([code; 10]);
            let folded = select(&person, 12.).unwrap().unwrap();
            assert!(!folded.predicted);
            assert_eq!(folded.native.fields_3c, [u32::from(code); 10]);
        }
        let mut wave = [0x24; 10];
        wave[2] = 0x30;
        person.poses = Some(wave);
        assert_eq!(
            select(&person, 12.).unwrap().unwrap().native.fields_3c,
            wave.map(u32::from)
        );
        person.poses = Some([0x30; 10]);
        assert!(select(&person, 12.).is_err());
        person.poses = Some([0x1f; 10]);
        assert!(select(&person, 12.).is_err());
        person.poses = Some([0x24; 10]);
        person.pose_parameter = Some(1);
        assert!(select(&person, 0.).is_err());
        person.pose_parameter = Some(0);
        person.head_tilt = Some(0.5);
        assert_eq!(select(&person, 0.).unwrap().unwrap().head_angles, [0.5, 0.]);
        for (input, expected) in [
            (10., f32::from_bits(0x3fc9_0fdb)),
            (-10., f32::from_bits(0xbfc9_0fdb)),
            (-0.25, -0.25),
        ] {
            person.head_tilt = Some(input);
            assert_eq!(
                select(&person, 0.).unwrap().unwrap().head_angles,
                [expected, 0.]
            );
        }
        for input in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            person.head_tilt = Some(input);
            assert!(select(&person, 0.).is_err());
        }
    }
}
