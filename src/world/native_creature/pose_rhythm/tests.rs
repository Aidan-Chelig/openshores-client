use super::*;

#[test]
fn every_pose_class_preserves_timer_call_and_immediate_zero_distinction() {
    let sine_poses = [
        1, 2, 4, 17, 22, 23, 24, 25, 26, 28, 29, 30, 33, 34, 37, 38, 39, 41, 42, 43, 44, 45, 47,
        48, 49, 50, 51, 52, 53, 56, 75, 108, 136, 137,
    ];
    for pose in 0..=255 {
        let mut calls = 0;
        let value = native_pose_rhythm(pose, 0., 0.75, || {
            calls += 1;
            Ok(12345)
        })
        .unwrap();
        let sine = sine_poses.contains(&pose) || pose >= 160;
        assert_eq!(calls, usize::from(sine), "pose {pose}");
        assert_eq!(value, if sine { 0.75_f32.sin() } else { 0. });
    }
    for pose in [256, 0x8000_0000, u32::MAX] {
        assert_eq!(
            native_pose_rhythm(pose, 1., 0.75, || Ok(0)).unwrap(),
            0.75_f32.sin()
        );
    }
}

#[test]
fn all_frequency_cases_and_signed_reference_times_use_ordered_float_stages() {
    let cases = [
        (1, 1_f32),
        (4, 0.5),
        (24, 2.5),
        (25, 3.),
        (26, 0.3),
        (33, 0.35),
        (37, 2.),
        (42, 0.75),
        (43, 0.85),
        (49, 0.1),
        (108, 0.1),
        (160, 0.),
    ];
    for (pose, frequency) in cases {
        for millis in [
            i64::MIN,
            -60001,
            -12345,
            0,
            1,
            12345,
            59999,
            60000,
            60001,
            i64::MAX,
        ] {
            let fraction = (((millis as f64) % 60000.) / 60000.) as f32;
            let mut angle = fraction * 60.;
            angle *= 1.25;
            angle *= frequency;
            angle *= f32::from_bits(0x4049_0fdb);
            let expected = (angle + angle + 0.125).sin();
            assert_eq!(
                native_pose_rhythm(pose, 1.25, 0.125, || Ok(millis))
                    .unwrap()
                    .to_bits(),
                expected.to_bits()
            );
        }
    }
}

struct Timer {
    events: Vec<&'static str>,
    reference: i64,
}
impl NativePoseRhythmTimer for Timer {
    fn start(&mut self) -> io::Result<()> {
        self.events.push("start");
        Ok(())
    }
    fn msecs_since_reference(&mut self) -> io::Result<i64> {
        self.events.push("reference");
        Ok(self.reference)
    }
}

#[test]
fn dna_rate_selector_and_start_order_cover_every_byte() {
    for byte in 0..=255 {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[17] = byte;
        let mut timer = Timer {
            events: vec![],
            reference: 1234,
        };
        let result = native_dh_pose_rhythm(&dna, 1, 0.25, 2., &mut timer).unwrap();
        assert_eq!(timer.events, ["start", "reference"]);
        let rate = 2_f32 / (4_f32 / (f32::from(byte & 15) + 1.) + 1.);
        assert_eq!(
            result.to_bits(),
            native_pose_rhythm(1, rate, 0.25, || Ok(1234))
                .unwrap()
                .to_bits()
        );
        timer.events.clear();
        assert_eq!(
            native_dh_pose_rhythm(&dna, 0, f32::NAN, f32::NAN, &mut timer)
                .unwrap()
                .to_bits(),
            0_f32.to_bits()
        );
        assert_eq!(timer.events, ["start"]);
    }
}

#[test]
fn required_timer_errors_propagate_and_immediate_zero_does_not_sample() {
    let missing = || {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no reference timer",
        ))
    };
    assert_eq!(
        native_pose_rhythm(1, 1., 0., missing).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(
        native_pose_rhythm(0, f32::NAN, f32::NAN, missing).unwrap(),
        0.
    );
    assert!(
        native_pose_rhythm(160, f32::INFINITY, 0., || Ok(1))
            .unwrap()
            .is_nan()
    );
}

#[test]
fn dna_timer_start_failure_is_not_hidden_by_an_immediate_zero_pose() {
    struct MissingTimer;
    impl NativePoseRhythmTimer for MissingTimer {
        fn start(&mut self) -> io::Result<()> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "no native time domain",
            ))
        }
        fn msecs_since_reference(&mut self) -> io::Result<i64> {
            panic!("must not sample after failed start")
        }
    }
    let error =
        native_dh_pose_rhythm(&NativeDhDna([0; 24]), 0, 0., 1., &mut MissingTimer).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Unsupported);
}
