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

pub struct PreparedAvatarAnimation {
    body: NativeDpInitialization,
    hands: NativeDpHandCache<'static>,
    feet: NativeDpFootCache<'static>,
    surfaces: Vec<NativeDpPreparedAvatarSurface>,
    eye: NativeDpAvatarEye,
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
        let mut body = self.native.body;
        let mut hands = NativeDpHandCache::new(mount);
        let mut feet = NativeDpFootCache::new(mount);
        for gait in [NativeDpGait::Walk, NativeDpGait::Run] {
            body.pose_locomotion(gait, 1., 0, &mut hands, &mut feet, &mut Timer(0))?;
        }
        for code in [0x12, 0x15] {
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
        wave.fields_3c[2] = 0x30;
        wave.fields_3c[5] = 0x30;
        body.pose_locomotion_channels(&wave, 0., 0, &mut hands, &mut feet, &mut Timer(0))?;
        body.pose_neutral_standing(0, &mut hands, &mut feet)?;
        Ok(PreparedAvatarAnimation {
            body,
            hands: hands.into_cached_only(),
            feet: feet.into_cached_only(),
            surfaces: self.animation_surfaces,
            eye,
        })
    }
}
impl PreparedAvatarAnimation {
    pub fn sample_channels(
        &mut self,
        pose: &NativeDpBodyPoseState,
        speed: f32,
        reference_millis: i64,
    ) -> io::Result<AvatarAnimationFrame> {
        self.body.pose_locomotion_channels(
            pose,
            speed,
            0,
            &mut self.hands,
            &mut self.feet,
            &mut Timer(reference_millis),
        )?;
        self.frame()
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
