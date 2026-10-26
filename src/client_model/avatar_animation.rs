//! Owned, I/O-free normal-avatar pose sampling after background preparation.
use super::PreparedNormalAvatar;
use crate::{
    asset_mount::AssetMount,
    world::{
        native_body::*,
        native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer},
    },
};
use std::io;
mod render_adjustment;

pub struct PreparedAvatarAnimation {
    body: NativeDpInitialization,
    hands: NativeDpHandCache<'static>,
    feet: NativeDpFootCache<'static>,
    surfaces: Vec<NativeDpPreparedAvatarSurface>,
    eye: NativeDpAvatarEye,
    ground_clearances: AvatarGroundClearances,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarGroundClearances {
    pub standing: f32,
    pub crouched: f32,
}
pub struct AvatarAnimationFrame {
    pub surfaces: Vec<NativeDpPosedAvatarSurface>,
    pub eye: [f64; 3],
}
struct Timer(i64);
impl NativePoseRhythmTimer for Timer {
    fn start(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn msecs_since_reference(&mut self) -> io::Result<i64> {
        Ok(self.0)
    }
}
impl PreparedNormalAvatar {
    /// Warm every admitted locomotion dependency off-thread, then discard mount access.
    pub fn into_animation(self, mount: &AssetMount) -> io::Result<PreparedAvatarAnimation> {
        let eye = self.native.prepare_eye(false)?;
        let ground_clearances = AvatarGroundClearances {
            standing: self.native.stand_crouch_ground_clearance(mount, false)?,
            crouched: self.native.stand_crouch_ground_clearance(mount, true)?,
        };
        let mut body = self.native.body;
        let mut hands = NativeDpHandCache::new(mount);
        let mut feet = NativeDpFootCache::new(mount);
        for gait in [NativeDpGait::Walk, NativeDpGait::Run] {
            body.pose_locomotion(gait, 1., 0, &mut hands, &mut feet, &mut Timer(0))?;
        }
        for code in [
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x1c, 0x18, 0x19, 0x1b, 0x20, 0x21, 0x35,
        ] {
            body.pose_locomotion_channels(
                &NativeDpBodyPoseState {
                    fields_3c: [code; 10],
                    field_6c: false,
                },
                0.,
                0,
                &mut hands,
                &mut feet,
                &mut Timer(0),
            )?;
        }
        let mut wave = NativeDpBodyPoseState {
            fields_3c: [0x24; 10],
            field_6c: false,
        };
        for code in [0x30, 0x1f] {
            wave.fields_3c[2] = code;
            wave.fields_3c[5] = code;
            body.pose_locomotion_channels(&wave, 0., 0, &mut hands, &mut feet, &mut Timer(0))?;
        }
        body.pose_neutral_standing(0, &mut hands, &mut feet)?;
        Ok(PreparedAvatarAnimation {
            body,
            hands: hands.into_cached_only(),
            feet: feet.into_cached_only(),
            surfaces: self.animation_surfaces,
            eye,
            ground_clearances,
        })
    }
}
impl PreparedAvatarAnimation {
    /// Calculate native nominal prone clearance at the supplied established
    /// head angles, using only cached anatomy. This samples the skeleton, not
    /// draw corners; the next animation sample resets/composes its own pose.
    /// Adult, unguided, nonvehicle context only. Does not move the actor root.
    pub fn prone_ground_clearance(&mut self, head_angles: [f32; 2]) -> io::Result<f64> {
        let pose = NativeDpBodyPoseState {
            fields_3c: [0x20; 10],
            field_6c: false,
        };
        self.body.pose_channels_with_head_angles(
            &pose,
            head_angles,
            0.,
            0,
            &mut self.hands,
            &mut self.feet,
            &mut Timer(0),
        )?;
        self.body
            .adult_prone_ground_clearance(&pose, 0, &mut self.feet)
    }

    /// Immutable nominal adult Stand/Crouch collision metadata, not a root offset.
    pub fn ground_clearances(&self) -> AvatarGroundClearances {
        self.ground_clearances
    }
    pub fn sample_channels(
        &mut self,
        pose: &NativeDpBodyPoseState,
        speed: f32,
        reference_millis: i64,
    ) -> io::Result<AvatarAnimationFrame> {
        self.sample_channels_with_head_angles(pose, [0., 0.], speed, reference_millis)
    }
    /// Sample with established native body pitch/roll, not raw wire head tilt.
    /// No installation I/O or Creature angle/timer side effects occur here.
    pub fn sample_channels_with_head_angles(
        &mut self,
        pose: &NativeDpBodyPoseState,
        head_angles: [f32; 2],
        speed: f32,
        reference_millis: i64,
    ) -> io::Result<AvatarAnimationFrame> {
        self.body.pose_channels_with_head_angles(
            pose,
            head_angles,
            speed,
            0,
            &mut self.hands,
            &mut self.feet,
            &mut Timer(reference_millis),
        )?;
        let mut frame = self.frame()?;
        let general = pose.fields_3c[3];
        // Adult normal body, field3a=false, nonvehicle parent. The connected
        // renderer admits globe children; its source adapter rejects growth.
        let offset = if matches!(general, 0x20 | 0x21) {
            self.body.head_location(general, false)?.map(|v| -v)
        } else {
            [0.; 3]
        };
        frame.adjust_render_pose(general, offset);
        Ok(frame)
    }
    pub fn sample(
        &mut self,
        gait: Option<NativeDpGait>,
        speed: f32,
        reference_millis: i64,
    ) -> io::Result<AvatarAnimationFrame> {
        if let Some(gait) = gait {
            self.body.pose_locomotion(
                gait,
                speed,
                0,
                &mut self.hands,
                &mut self.feet,
                &mut Timer(reference_millis),
            )?;
        } else {
            self.body
                .pose_neutral_standing(0, &mut self.hands, &mut self.feet)?;
        }
        self.frame()
    }
    fn frame(&self) -> io::Result<AvatarAnimationFrame> {
        Ok(AvatarAnimationFrame {
            surfaces: self
                .surfaces
                .iter()
                .map(|surface| surface.skin(self.body.bones()))
                .collect::<io::Result<_>>()?,
            eye: self.eye.location(self.body.bones())?,
        })
    }
}
