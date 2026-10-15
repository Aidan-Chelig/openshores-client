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
}
