//! Opt-in mounted-body pose Slither composition; no target execution/network.
use openshores_client::{
    asset_mount::AssetMount,
    world::{
        native_body::{NativeDpHandCache, NativeDpInitialization, NativeDpPartKind},
        native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer, native_dh_pose_rhythm},
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

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn mounted_body_pose_slither_composes_actual_resource_initialized_parts() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut cache = NativeDpHandCache::new(&mount);
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[14] = 0xb0;
    let mut body = NativeDpInitialization::begin(&dna);
    body.finish_part_calculations(0, &mut cache).unwrap();
    let mut pose = NativeDpBodyPoseState::default();
    pose.fields_3c[3] = 24;
    pose.fields_3c[7] = 17;
    let mut timer = Timer::default();
    let amplitude = native_dh_pose_rhythm(&dna, 17, f32::from_bits(0x40490fdb), 0.75, &mut timer)
        .unwrap()
        * f32::from_bits(0x3f060a92);
    let tail = body
        .parts()
        .iter()
        .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
        .unwrap();
    let mut expected = body.bones().clone();
    tail.slither_tail(&mut expected, body.dimensions(), 0., amplitude)
        .unwrap();
    timer.events.clear();
    body.slither_tail_pose(&pose, pose.fields_3c[7], 0.75, &mut timer)
        .unwrap();
    assert_eq!(*body.bones(), expected);
    assert_eq!(timer.events, ["start", "sample"]);
    for flag in [false, true] {
        let mut expected = body.bones().clone();
        body.parts()
            .iter()
            .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
            .unwrap()
            .fold_tail(&mut expected, body.dimensions(), flag)
            .unwrap();
        body.fold_tail(flag).unwrap();
        assert_eq!(*body.bones(), expected);
        assert!(
            body.bones()
                .slots
                .iter()
                .all(|b| b.matrix_00.iter().all(|v| v.is_finite()))
        );
    }
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn mounted_body_rhythmic_tail_motions_and_relaxation_compose_real_bones() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut cache = NativeDpHandCache::new(&mount);
    for form in [0, 3, 6] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 1;
        dna.0[14] = 0xb0;
        dna.0[16] = form;
        let mut body = NativeDpInitialization::begin(&dna);
        body.finish_part_calculations(0, &mut cache).unwrap();
        let mut pose = NativeDpBodyPoseState::default();
        pose.fields_3c[3] = 17;
        for (motion, word, arc, speed) in [
            (0, 17, 0x3eb2b8c2, 1.),
            (1, 17, 0x3e567750, 0.75),
            (2, 30, 0x3f060a92, 0.75),
            (3, 47, 0x3e860a92, 1.),
        ] {
            let mut timer = Timer::default();
            let amplitude = (f32::from_bits(arc) / 4.)
                * native_dh_pose_rhythm(&dna, word, f32::from_bits(0x40490fdb), speed, &mut timer)
                    .unwrap();
            let mut expected = body.bones().clone();
            let tail = body
                .parts()
                .iter()
                .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
                .unwrap();
            if amplitude != 0. {
                let (pitch, yaw) = if tail.tail_can_bend_laterally().unwrap() {
                    (
                        if motion == 3 && tail.tail_can_bend_vertically().unwrap() {
                            -f32::from_bits(0x3e860a92)
                        } else {
                            0.
                        },
                        amplitude,
                    )
                } else {
                    (amplitude, 0.)
                };
                tail.bend_tail(&mut expected, body.dimensions(), pitch, yaw, -1.)
                    .unwrap();
            }
            let events = timer.events.clone();
            timer.events.clear();
            match motion {
                0 => body.swim_tail(&pose, &mut timer),
                1 => body.wave_tail(&pose, -1., speed, &mut timer),
                2 => body.lash_tail(-1., speed, &mut timer),
                _ => body.wag_tail(-1., &mut timer),
            }
            .unwrap();
            assert_eq!(*body.bones(), expected);
            assert_eq!(timer.events, events);
        }
        for clearance in [-1., 0., 1., f32::NAN] {
            let mut expected = body.bones().clone();
            let tail = body
                .parts()
                .iter()
                .find(|p| p.allocation.kind == NativeDpPartKind::Tail)
                .unwrap();
            if tail.tail_can_bend_vertically().unwrap() && clearance > 0. {
                tail.bend_tail(
                    &mut expected,
                    body.dimensions(),
                    f32::from_bits(0x3e860a92),
                    0.,
                    clearance,
                )
                .unwrap();
            } else if tail.tail_can_bend_laterally().unwrap() {
                tail.slither_tail(
                    &mut expected,
                    body.dimensions(),
                    0.,
                    f32::from_bits(0x3e32b8c2),
                )
                .unwrap();
            }
            body.relax_tail(clearance).unwrap();
            assert_eq!(*body.bones(), expected);
        }
    }
}
