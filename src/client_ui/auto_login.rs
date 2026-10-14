//! Authentication only: no scripted movement or inventory actions.
use super::*;

#[derive(Resource)]
pub(super) struct Workflow {
    profile: LocalAutoLogin,
    stage: u8,
    started: std::time::Instant,
    bounded: bool,
}

impl Workflow {
    pub(super) fn new(profile: LocalAutoLogin, bounded: bool) -> Self {
        Self {
            profile,
            stage: 0,
            started: std::time::Instant::now(),
            bounded,
        }
    }
}

pub(super) fn workflow(
    mut flow: ResMut<Workflow>,
    mut ui: ResMut<Presentation>,
    bridge: Res<Bridge>,
    mut exit: MessageWriter<AppExit>,
) {
    if flow.stage == 3 {
        return;
    }
    if flow.stage == 0 && !ui.assets_ready {
        flow.started = std::time::Instant::now();
        if flow.bounded && ui.asset_status.starts_with("Asset preparation failed:") {
            eprintln!("{}", ui.asset_status);
            exit.write(AppExit::error());
        }
        return;
    }
    if flow.started.elapsed().as_secs() >= 45 {
        flow.stage = 3;
        if flow.bounded {
            exit.write(AppExit::error());
        }
        return;
    }
    match flow.stage {
        0 => {
            ui.username = flow.profile.username.clone();
            ui.password = std::mem::take(&mut flow.profile.password);
            activate(Action::Login, &mut ui, &bridge);
            flow.stage = 1;
        }
        1 if ui.model.phase == Phase::Roster => {
            activate(Action::Join(flow.profile.avatar_id), &mut ui, &bridge);
            flow.stage = 2;
        }
        2 if ui.model.phase == Phase::Playing => {
            println!("Quick-dev connected to avatar {}", flow.profile.avatar_id);
            flow.stage = 3;
        }
        _ => {}
    }
}
