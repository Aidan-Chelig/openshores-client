//! Bounded end-to-end GUI scenario, opt-in for owned disposable test accounts.
use super::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Resource)]
pub(super) struct Workflow {
    profile: LocalSmoke,
    stage: u8,
    started: Instant,
    entered: Instant,
    target: Option<u32>,
    hunger: u16,
    complete: Arc<AtomicBool>,
}
impl Workflow {
    pub(super) fn new(profile: LocalSmoke, complete: Arc<AtomicBool>) -> Self {
        Self {
            profile,
            stage: 0,
            started: Instant::now(),
            entered: Instant::now(),
            target: None,
            hunger: 0,
            complete,
        }
    }
    fn advance(&mut self, stage: u8) {
        self.stage = stage;
        self.entered = Instant::now();
    }
}
fn has_gear(ui: &Presentation, slot: u8, sub: u8, cid: i16) -> bool {
    ui.model.gear().is_some_and(|gear| {
        gear.slots.iter().any(|entry| {
            entry.slot_type == slot
                && entry.subindex == sub
                && entry
                    .item
                    .as_ref()
                    .is_some_and(|item| item.common.commodity_id == cid)
        })
    })
}
fn hunger(ui: &Presentation) -> u16 {
    ui.model
        .avatar
        .and_then(|id| ui.model.world.entities.get(&id))
        .and_then(|entity| entity.person.as_ref())
        .and_then(|person| person.hunger_raw)
        .unwrap_or(0)
        & 0x7fff
}

pub(super) fn workflow(
    mut workflow: ResMut<Workflow>,
    mut ui: ResMut<Presentation>,
    bridge: Res<Bridge>,
    mut commands: Commands,
    terrain: Query<Entity, With<TerrainEntity>>,
    mut exit: MessageWriter<AppExit>,
) {
    if workflow.started.elapsed() > Duration::from_secs(55)
        || ui.model.status.starts_with("Error:")
        || ui.model.status.starts_with("Invalid server")
    {
        eprintln!(
            "GUI slice failed at stage {}: {}",
            workflow.stage, ui.model.status
        );
        exit.write(AppExit::Error(std::num::NonZeroU8::new(1).unwrap()));
        return;
    }
    match workflow.stage {
        0 => {
            ui.username = workflow.profile.username.clone();
            ui.password = std::mem::take(&mut workflow.profile.password);
            activate(Action::Login, &mut ui, &bridge);
            workflow.advance(1);
        }
        1 if ui.model.phase == Phase::Roster => {
            assert_eq!(ui.model.roster.as_ref().unwrap().len(), 6);
            activate(Action::Join(workflow.profile.avatar_id), &mut ui, &bridge);
            workflow.advance(2);
        }
        2 if ui.model.phase == Phase::Playing
            && workflow.entered.elapsed() > Duration::from_secs(2) =>
        {
            assert_eq!(ui.model.world.commodities.len(), 615);
            assert!(ui.terrain.is_some() && terrain.iter().count() == 1);
            assert!(has_gear(&ui, 5, 0, 1));
            activate(Action::SelectGear(5, 0), &mut ui, &bridge);
            activate(Action::Move, &mut ui, &bridge);
            workflow.advance(3);
        }
        3 if has_gear(&ui, 1, 0, 1) && !has_gear(&ui, 5, 0, 1) => {
            activate(Action::SelectGear(1, 0), &mut ui, &bridge);
            activate(Action::Drop, &mut ui, &bridge);
            workflow.advance(4);
        }
        4 => {
            let target = ui
                .model
                .visible_items()
                .find(|item| {
                    item.item
                        .as_ref()
                        .and_then(|state| state.value.as_ref())
                        .and_then(|value| value.common.as_ref())
                        .is_some_and(|item| item.commodity_id == 1)
                })
                .map(|item| item.id);
            if let Some(target) = target {
                workflow.target = Some(target);
                activate(Action::SelectItem(target), &mut ui, &bridge);
                activate(Action::Pickup, &mut ui, &bridge);
                workflow.advance(5);
            }
        }
        5 if has_gear(&ui, 1, 0, 1)
            && workflow
                .target
                .is_some_and(|target| !ui.model.world.visible_ids.contains(&target)) =>
        {
            assert!(has_gear(&ui, 5, 3, 15));
            workflow.hunger = hunger(&ui);
            activate(Action::SelectGear(5, 3), &mut ui, &bridge);
            activate(Action::Use, &mut ui, &bridge);
            workflow.advance(6);
        }
        6 if !has_gear(&ui, 5, 3, 15) && hunger(&ui) > workflow.hunger => {
            assert!(has_gear(&ui, 4, 0, 15));
            let mut projected = ui.model.position().unwrap();
            projected[0] += 2.0;
            let rotation = ui.model.rotation().unwrap();
            ui.local_position = Some(projected);
            send(
                &mut ui,
                &bridge,
                Request::Scene(SceneCommand::PositionReport {
                    projected,
                    rotation,
                }),
            );
            ui.hand = true;
            ui.top_down = true;
            ui.top_height = 7.;
            ui.dirty = true;
            workflow.advance(7);
        }
        7 if ui.model.reported_position.is_some()
            && workflow.entered.elapsed() > Duration::from_secs(2) =>
        {
            if let Some(path) = &workflow.profile.screenshot {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path.clone()));
            }
            workflow.advance(8);
        }
        8 if workflow.entered.elapsed() > Duration::from_secs(2) => {
            assert_eq!(ui.model.gear().unwrap().declared_slots, 6);
            assert_eq!(ui.model.world.commodities.len(), 615);
            println!(
                "GUI slice: six-slot login, live terrain, gear move/drop/pickup/use, received hunger and raw position report passed"
            );
            activate(Action::Disconnect, &mut ui, &bridge);
            workflow.advance(9);
        }
        9 if ui.model.phase == Phase::Disconnected => {
            workflow.complete.store(true, Ordering::Release);
            println!("GUI slice: clean disconnect passed");
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
