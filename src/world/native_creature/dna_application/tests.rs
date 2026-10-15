use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Snapshot,
    CreateAu(u32, u32, u32),
    UpdateAu(u32),
    CreateDp(u8, bool),
    UpdateDp(u32),
    Hp(NativeCreatureBodyKind, i16),
    DestroyDp,
    DestroyAu,
    Pose([u32; 10], bool, u32),
    Timer(NativeDnaTimer),
}
struct Context {
    old: NativeDnaModelSnapshot,
    events: Vec<Event>,
    fail_at: Option<usize>,
}
impl Context {
    fn new(old: NativeDnaModelSnapshot) -> Self {
        Self {
            old,
            events: vec![],
            fail_at: None,
        }
    }
    fn event(&mut self, event: Event) -> io::Result<()> {
        self.events.push(event);
        if self.fail_at == Some(self.events.len() - 1) {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        } else {
            Ok(())
        }
    }
}
// Trace-only model operations prove outer ordering/parameters, not morphology.
impl NativeDnaModelContext for Context {
    fn snapshot(&mut self) -> io::Result<NativeDnaModelSnapshot> {
        self.event(Event::Snapshot)?;
        Ok(self.old)
    }
    fn create_animalia(&mut self, _: &NativeAuDna, a: u32, b: u32, c: u32) -> io::Result<()> {
        self.event(Event::CreateAu(a, b, c))
    }
    fn update_animalia(&mut self, _: &NativeAuDna, a: u32) -> io::Result<()> {
        self.event(Event::UpdateAu(a))
    }
    fn create_dp(&mut self, _: &NativeDhDna, a: u8, b: bool) -> io::Result<()> {
        self.event(Event::CreateDp(a, b))
    }
    fn update_dp(&mut self, _: &NativeDhDna, scale: f32) -> io::Result<()> {
        self.event(Event::UpdateDp(scale.to_bits()))
    }
    fn set_hit_points(&mut self, kind: NativeCreatureBodyKind, hp: i16) -> io::Result<()> {
        self.event(Event::Hp(kind, hp))
    }
    fn destroy_dp(&mut self) -> io::Result<()> {
        self.event(Event::DestroyDp)
    }
    fn destroy_animalia(&mut self) -> io::Result<()> {
        self.event(Event::DestroyAu)
    }
    fn animalia_pose(&mut self, fields: [u32; 10], mode: bool, speed: f32) -> io::Result<()> {
        self.event(Event::Pose(fields, mode, speed.to_bits()))
    }
    fn restart_timer(&mut self, timer: NativeDnaTimer) -> io::Result<()> {
        self.event(Event::Timer(timer))
    }
}
fn au() -> NativeDna {
    NativeDna::Au(NativeAuDna {
        version: 9,
        chromatids: [vec![], vec![]],
    })
}
fn dh() -> NativeDna {
    NativeDna::Dh(NativeDhDna([0; 24]))
}
fn dp() -> NativeDpDnaPrior {
    NativeDpDnaPrior {
        byte_3a: 0,
        byte_64: 0,
        hit_points: 0,
        general_pose_48: 0x8b,
    }
}
fn animal() -> NativeAnimaliaDnaPrior {
    NativeAnimaliaDnaPrior {
        chromatid_08: 9,
        handedness_2c: u32::MAX,
        hit_points: 30,
        general_pose_6c: 0x30,
    }
}

#[test]
fn animalia_raw_metadata_is_preserved_and_dp_constructor_parameters_compare_to_zero() {
    for chromatid in [0, 1, 255, u32::MAX] {
        for handedness in [0, 1, 255, u32::MAX] {
            let old = NativeDnaModelSnapshot {
                animalia: Some(NativeAnimaliaDnaPrior {
                    chromatid_08: chromatid,
                    handedness_2c: handedness,
                    ..animal()
                }),
                ..Default::default()
            };
            let mut context = Context::new(old);
            let _report = native_apply_creature_dna(&au(), &mut context).unwrap();
            assert_eq!(context.events[1], Event::UpdateAu(chromatid));
            let mut context = Context::new(old);
            let _report = native_apply_creature_dna(&dh(), &mut context).unwrap();
            assert_eq!(
                context.events[1],
                Event::CreateDp(u8::from(chromatid == 0), handedness == 0)
            );
        }
    }
}

#[test]
fn every_raw_dp_configuration_uses_native_comparisons_not_boolean_guessing() {
    for a in 0..=u8::MAX {
        for b in 0..=u8::MAX {
            let old = NativeDnaModelSnapshot {
                dp: Some(NativeDpDnaPrior {
                    byte_3a: a,
                    byte_64: b,
                    ..dp()
                }),
                ..Default::default()
            };
            let mut context = Context::new(old);
            let report = native_apply_creature_dna(&au(), &mut context).unwrap();
            assert!(report.created && report.removed_other_body && report.pose_reinitialized);
            assert_eq!(
                context.events,
                [
                    Event::Snapshot,
                    Event::CreateAu(u32::from(b != 1), u32::from(a ^ 1), 1),
                    Event::Hp(NativeCreatureBodyKind::Animalia, 0),
                    Event::DestroyDp,
                    Event::Timer(NativeDnaTimer::Field5c8),
                    Event::Pose(
                        [0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x8b],
                        false,
                        1_f32.to_bits()
                    ),
                    Event::Timer(NativeDnaTimer::Field630)
                ]
            );
        }
    }
}

#[test]
fn body_priority_reuse_creation_defaults_and_hp_clamp_follow_both_native_branches() {
    for has_dp in [false, true] {
        for has_au in [false, true] {
            for hp in [i16::MIN, -31, -30, 0, 1, i16::MAX] {
                for growth in [0, 1, 255] {
                    let old = NativeDnaModelSnapshot {
                        dp: has_dp.then_some(NativeDpDnaPrior {
                            hit_points: hp,
                            ..dp()
                        }),
                        animalia: has_au.then_some(NativeAnimaliaDnaPrior {
                            hit_points: hp,
                            ..animal()
                        }),
                        creature_byte_494: growth,
                    };
                    let expected_hp = if has_dp || has_au { hp.max(-30) } else { 0 };
                    for dna in [au(), dh()] {
                        let mut context = Context::new(old);
                        let report = native_apply_creature_dna(&dna, &mut context).unwrap();
                        let mut expected = vec![Event::Snapshot];
                        if matches!(dna, NativeDna::Au(_)) {
                            assert_eq!(report.kind, NativeCreatureBodyKind::Animalia);
                            assert_eq!(report.created, !has_au);
                            assert_eq!(report.removed_other_body, has_dp);
                            if has_au {
                                expected.push(Event::UpdateAu(9));
                            } else {
                                expected.push(Event::CreateAu(
                                    u32::from(has_dp),
                                    1,
                                    u32::from(growth == 0),
                                ));
                            }
                            expected.push(Event::Hp(NativeCreatureBodyKind::Animalia, expected_hp));
                            if has_dp {
                                expected.push(Event::DestroyDp);
                            }
                            expected.push(Event::Timer(NativeDnaTimer::Field5c8));
                            let general = if has_au {
                                0x30
                            } else if has_dp {
                                0x8b
                            } else {
                                0x24
                            };
                            let fields = if general == 0x8b {
                                [0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x8b]
                            } else {
                                [
                                    general, general, general, general, general, general, general,
                                    general, general, 0x8c,
                                ]
                            };
                            expected.push(Event::Pose(fields, false, 1_f32.to_bits()));
                            expected.push(Event::Timer(NativeDnaTimer::Field630));
                        } else {
                            assert_eq!(report.kind, NativeCreatureBodyKind::DpBody);
                            assert_eq!(report.created, !has_dp);
                            assert_eq!(report.removed_other_body, has_au);
                            assert!(!report.pose_reinitialized);
                            if has_dp {
                                expected.push(Event::UpdateDp(1_f32.to_bits()));
                            } else {
                                expected.push(Event::CreateDp(u8::from(!has_au), false));
                            }
                            expected.push(Event::Hp(NativeCreatureBodyKind::DpBody, expected_hp));
                            if has_au {
                                expected.push(Event::DestroyAu);
                            }
                            expected.push(Event::Timer(NativeDnaTimer::Field5c8));
                        }
                        assert_eq!(context.events, expected);
                        for fail in 0..expected.len() {
                            let mut context = Context::new(old);
                            context.fail_at = Some(fail);
                            assert_eq!(
                                native_apply_creature_dna(&dna, &mut context)
                                    .unwrap_err()
                                    .kind(),
                                io::ErrorKind::Unsupported
                            );
                            assert_eq!(context.events, expected[..=fail]);
                        }
                    }
                }
            }
        }
    }
}
