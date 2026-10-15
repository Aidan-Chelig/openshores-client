//! Mounted torso dispatch composed with the real body-wide bone operations.
use openshores_client::{
    asset_mount::AssetMount,
    world::{
        native_body::{NativeDpHandCache, NativeDpInitialization, native_dh_waist_vertex},
        native_creature::{NativeDpBodyPoseState, NativePoseRhythmTimer},
        native_dna::NativeDhDna,
    },
};
use std::io;

#[derive(Default)]
struct Timer {
    starts: usize,
    samples: usize,
    fail: bool,
}
impl NativePoseRhythmTimer for Timer {
    fn start(&mut self) -> io::Result<()> {
        self.starts += 1;
        if self.fail {
            Err(io::ErrorKind::Unsupported.into())
        } else {
            Ok(())
        }
    }
    fn msecs_since_reference(&mut self) -> io::Result<i64> {
        self.samples += 1;
        Ok(0)
    }
}

fn offset(role: &str, base: f32, general: u32) -> f32 {
    let nonzero = base != 0.;
    match role {
        "none" | "42" | "43" | "tiny" => 0.,
        "fixed" => f32::from_bits(0x3fc25c20),
        "fallback" | "small" => {
            if nonzero {
                base
            } else {
                f32::from_bits(0x3fbde44f)
            }
        }
        "negative" | "50" => {
            if nonzero {
                base
            } else {
                -0.17453292
            }
        }
        "quarter" => f32::from_bits(0x3fb2b8c3),
        "halfpi" => std::f32::consts::FRAC_PI_2,
        "23" => {
            if nonzero {
                base
            } else {
                0.2617994
            }
        }
        "24" => {
            if nonzero {
                0.2617994
            } else {
                0.13962634
            }
        }
        "25" => {
            if nonzero {
                0.08726646
            } else {
                0.017453292
            }
        }
        "27" => {
            if nonzero {
                0.43633232
            } else {
                0.2617994
            }
        }
        "28" => {
            if nonzero {
                base * 0.5
            } else {
                std::f32::consts::FRAC_PI_4
            }
        }
        "29" => f32::from_bits(0x3fcb4bc4),
        "37" => {
            if nonzero {
                base
            } else {
                f32::from_bits(0x3fa78d36)
            }
        }
        "41" => match general {
            24 | 25 | 27 | 53 => base - 0.17453292,
            42 | 43 | 45 => 0.,
            _ => std::f32::consts::FRAC_PI_4,
        },
        "45" => {
            if nonzero {
                0.
            } else {
                f32::from_bits(0x3fc25c20)
            }
        }
        "46" => f32::from_bits(0x3fc49809),
        _ => panic!("unknown fixture role {role}"),
    }
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn mounted_all_torso_poses_use_real_waist_transforms_and_required_resources() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut hands = NativeDpHandCache::new(&mount);
    for class in [0, 0x80, 0x60] {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[1] = 3;
        dna.0[2] = 2;
        dna.0[5] = 2;
        dna.0[11] = class;
        dna.0[12] = 12;
        dna.0[14] = 0x31;
        dna.0[20] = 0x60;
        let mut template = NativeDpInitialization::begin(&dna);
        template.finish_part_calculations(-7, &mut hands).unwrap();
        assert_eq!(template.parts().len(), 16);
        assert_ne!(native_dh_waist_vertex(&dna), 0);
        for height in [0., 0.25, 1.] {
            let base = hands
                .torso_base_angle(&dna, template.dimensions(), -7, height)
                .unwrap();
            for line in include_str!("fixtures/native_dp_position_torso.tsv")
                .lines()
                .filter(|l| !l.starts_with('#'))
            {
                let (selected, role) = line.split_once('\t').unwrap();
                let selected: u32 = selected.parse().unwrap();
                for general in [0, 24, 42, 53, 0x1002a] {
                    let mut pose = NativeDpBodyPoseState::default();
                    pose.fields_3c[8] = selected;
                    pose.fields_3c[3] = general;
                    let mut expected = template.clone();
                    let proposed = offset(role, base, general);
                    let applied = expected.rotate_waist(proposed).unwrap();
                    let mut actual = template.clone();
                    let returned = actual
                        .position_torso(&pose, height, 0.375, -7, &mut hands, &mut Timer::default())
                        .unwrap();
                    assert_eq!(
                        returned.to_bits(),
                        if applied { proposed.to_bits() } else { 0 },
                        "pose {selected}, class {class}, height {height}, general {general}"
                    );
                    assert_eq!(actual.bones(), expected.bones());
                }
            }
        }
        let mut pose = NativeDpBodyPoseState::default();
        pose.fields_3c[8] = 17;
        let before = template.bones().clone();
        let mut timer = Timer {
            fail: true,
            ..Default::default()
        };
        assert_eq!(
            template
                .position_torso(&pose, 0., 1., -7, &mut hands, &mut timer)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(template.bones(), &before);
        assert_eq!((timer.starts, timer.samples), (1, 0));
    }
    // Rhythm is evaluated before the zero-waist guard, even when no rotation
    // can apply. Native default poses, in contrast, never start a timer.
    let mut body = NativeDpInitialization::begin(&NativeDhDna([0; 24]));
    let before = body.bones().clone();
    let mut pose = NativeDpBodyPoseState::default();
    pose.fields_3c[8] = 42;
    let mut timer = Timer::default();
    assert_eq!(
        body.position_torso(&pose, 0., 1., 0, &mut hands, &mut timer)
            .unwrap(),
        0.
    );
    assert_eq!((timer.starts, timer.samples), (1, 1));
    assert_eq!(body.bones(), &before);
    pose.fields_3c[8] = 0x1002a;
    body.position_torso(&pose, 0., 1., 0, &mut hands, &mut timer)
        .unwrap();
    assert_eq!((timer.starts, timer.samples), (1, 1));
}
