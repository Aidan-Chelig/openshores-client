use super::*;

/// Fresh-value InitPoses: animation-only8b..9f goes into the last word with
/// nine standing24 words; all other general words fill the first nine and
/// use final8c. The observed SitAtConsole/SetDNA callers use separate storage.
pub fn native_animalia_initialize_poses(general_word: u32) -> [u32; 10] {
    let animation_only = general_word.wrapping_sub(0x8b) < 0x15;
    let mut fields = [if animation_only { 0x24 } else { general_word }; 10];
    fields[9] = if animation_only { general_word } else { 0x8c };
    fields
}

/// Exact body block prepared by SitAtConsole after computing its general pose.
pub fn native_station_body_pose(kind: NativeCreatureBodyKind, general_word: u32) -> [u32; 10] {
    match kind {
        NativeCreatureBodyKind::DpBody => [general_word; 10],
        NativeCreatureBodyKind::Animalia => native_animalia_initialize_poses(general_word),
    }
}

/// CurrentGeneralPose prioritizes DpBody word at48 (array index3), otherwise
/// Animalia word at6c (index0). Missing both is a Rust safety error instead of
/// the native null dereference. Raw wide words are not normalized here.
pub fn native_current_general_pose(
    dp: Option<&NativeDpBodyPoseState>,
    animal: Option<&NativeAnimaliaPoseState>,
) -> io::Result<u32> {
    dp.map(|value| value.fields_3c[3])
        .or_else(|| animal.map(|value| value.fields_6c[0]))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "CurrentGeneralPose requires a selected native body",
            )
        })
}
