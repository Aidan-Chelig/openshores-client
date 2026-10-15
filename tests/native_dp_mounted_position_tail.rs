//! Opt-in complete PositionTail composition using native jump-table fixture.
use openshores_client::{
    asset_mount::AssetMount,
    world::{
        native_body::{NativeDpHandCache, NativeDpInitialization},
        native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer},
        native_dna::NativeDhDna,
    },
};
use std::io;
#[derive(Default)]
struct Timer {
    events: Vec<&'static str>,
}
impl NativePoseRhythmTimer for Timer {
    fn start(&mut self) -> io::Result<()> {
        self.events.push("start");
        Ok(())
    }
    fn msecs_since_reference(&mut self) -> io::Result<i64> {
        self.events.push("sample");
        Ok(12345)
    }
}

fn expected(
    body: &mut NativeDpInitialization,
    pose: &NativeDpBodyPoseState,
    action: &str,
    legs: bool,
    clearance: f32,
    flag: bool,
    timer: &mut Timer,
) -> io::Result<()> {
    let special = [42, 43, 45].contains(&pose.fields_3c[3]);
    let adjusted = if special { -1. } else { clearance };
    match action {
        "relax_or_slither" => {
            if legs {
                body.relax_tail(clearance)
            } else {
                body.slither_tail_pose(pose, pose.fields_3c[7], 1., timer)
            }
        }
        "wave_or_slither" => {
            if legs {
                body.wave_tail(pose, clearance, 0.75, timer)
            } else {
                body.slither_tail_pose(pose, pose.fields_3c[7], 0.75, timer)
            }
        }
        "wave_or_fixed_slither" => {
            if legs {
                body.wave_tail(pose, clearance, 0.75, timer)
            } else {
                body.slither_tail_pose(pose, pose.fields_3c[7], 1., timer)
            }
        }
        "lash_or_slither" => {
            if legs {
                body.lash_tail(clearance, 0.75, timer)
            } else {
                body.slither_tail_pose(pose, pose.fields_3c[7], 0.75, timer)
            }
        }
        "receptive" => body.receptive_tail_facing_up(timer),
        "dead_down" => body.dead_tail_facing_down(),
        "dead_left" => body.dead_tail_facing_left(),
        "wag" => body.wag_tail(adjusted, timer),
        "fold" => body.fold_tail(flag),
        "lash" => body.lash_tail(adjusted, 1., timer),
        "strike" => body.strike_tail(pose, adjusted, timer),
        "swim" => body.swim_tail(pose, timer),
        "writhe_down" => body.writhe_tail_facing_down(timer),
        "writhe_left" => body.writhe_tail_facing_left(timer),
        "writhe_right" => body.writhe_tail_facing_right(timer),
        "writhe_up" => body.writhe_tail_facing_up(timer),
        "noop" => Ok(()),
        _ => panic!("unknown native table target"),
    }
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn complete_position_tail_matches_native_dispatch_table_on_mounted_bodies() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut cache = NativeDpHandCache::new(&mount);
    for legs in [false, true] {
        for (shape, form) in [(0, 0), (3, 0), (3, 3), (3, 6)] {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[1] = 1;
            dna.0[12] = if legs { 4 } else { 0 };
            dna.0[14] = 0x80 | shape << 4;
            dna.0[16] = form;
            let mut initialized = NativeDpInitialization::begin(&dna);
            initialized.finish_part_calculations(0, &mut cache).unwrap();
            for line in include_str!("fixtures/native_dp_tail_dispatch.tsv")
                .lines()
                .filter(|l| !l.starts_with('#'))
            {
                let (word, action) = line.split_once('\t').unwrap();
                let word = word.parse().unwrap();
                for general in [0, 42, 43, 45, 0x1002a] {
                    for flag in [false, true] {
                        for clearance in [-1., 0., f32::NAN] {
                            let mut pose = NativeDpBodyPoseState::default();
                            pose.fields_3c[3] = general;
                            pose.fields_3c[7] = word;
                            let mut actual = initialized.clone();
                            let mut control = initialized.clone();
                            let mut actual_timer = Timer::default();
                            let mut expected_timer = Timer::default();
                            expected(
                                &mut control,
                                &pose,
                                action,
                                legs,
                                clearance,
                                flag,
                                &mut expected_timer,
                            )
                            .unwrap();
                            actual
                                .position_tail(&pose, clearance, 0.75, flag, &mut actual_timer)
                                .unwrap();
                            assert_eq!(
                                actual.bones(),
                                control.bones(),
                                "tail {word}, general {general}, legs {legs}, form {form}, flag {flag}, clearance {clearance}"
                            );
                            assert_eq!(actual_timer.events, expected_timer.events);
                        }
                    }
                }
            }
        }
    }
}
