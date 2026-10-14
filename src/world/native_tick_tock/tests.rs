use super::*;
use std::collections::VecDeque;

#[derive(Default)]
struct Probe {
    blocked: bool,
    ticker: bool,
    interval: u32,
    parents: VecDeque<Option<SolarSystem>>,
    calls: Vec<&'static str>,
    applied: Vec<RegistrationAction>,
    interval_after_ticker: Option<u32>,
    field_after_ticker: Option<u8>,
}
impl Context for Probe {
    fn must_not_tick_tock(&mut self) -> bool {
        self.calls.push("blocked");
        self.blocked
    }
    fn is_ticker(&mut self) -> bool {
        self.calls.push("ticker");
        self.ticker
    }
    fn is_tocker(&mut self) -> u32 {
        self.calls.push("tocker");
        self.interval
    }
    fn find_parent_solar_system(&mut self) -> Option<SolarSystem> {
        self.calls.push("parent");
        self.parents.pop_front().expect("unexpected parent lookup")
    }
    fn atom_type(&mut self) -> u8 {
        self.calls.push("type");
        0x16 + self.applied.len() as u8
    }
    fn is_ticking_contents(&mut self, solar: SolarSystem) -> bool {
        solar.ticking_contents
    }
    fn solar_field_3a9(&mut self, solar: SolarSystem) -> u8 {
        if self.applied.is_empty() {
            solar.field_3a9
        } else {
            self.field_after_ticker.unwrap_or(solar.field_3a9)
        }
    }
}

impl RegistrationContext for Probe {
    fn apply_registration(&mut self, action: RegistrationAction) {
        self.calls.push(match action {
            RegistrationAction::ReleaseTickTock { .. } => "release-both",
            RegistrationAction::ReleaseTicker { .. } => "release-ticker",
            RegistrationAction::ReleaseTocker { .. } => "release-tocker",
            RegistrationAction::UpdateOrAddTicker { .. } => "update-ticker",
            RegistrationAction::UpdateOrAddTocker { .. } => "update-tocker",
        });
        if matches!(
            action,
            RegistrationAction::ReleaseTicker { .. } | RegistrationAction::UpdateOrAddTicker { .. }
        ) && let Some(interval) = self.interval_after_ticker
        {
            self.interval = interval;
        }
        self.applied.push(action);
    }
}

#[test]
fn inline_effects_are_visible_to_later_getters_and_are_not_deferred() {
    let system = SolarSystem {
        id: 9,
        ticking_contents: true,
        field_3a9: 0,
    };
    let mut p = Probe {
        ticker: true,
        parents: [Some(system)].into(),
        interval_after_ticker: Some(u32::MAX),
        ..Probe::default()
    };
    apply_registrations(7, &mut p);
    assert_eq!(
        p.calls,
        [
            "blocked",
            "ticker",
            "parent",
            "type",
            "update-ticker",
            "tocker",
            "type",
            "update-tocker"
        ]
    );
    assert_eq!(
        p.applied,
        [
            RegistrationAction::UpdateOrAddTicker {
                id: 7,
                atom_type: 0x16,
                solar_id: 9
            },
            RegistrationAction::UpdateOrAddTocker {
                id: 7,
                atom_type: 0x17,
                solar_id: 9,
                raw_interval: u32::MAX
            }
        ]
    );
    let mut p = Probe {
        interval: 123,
        interval_after_ticker: Some(0),
        ..Probe::default()
    };
    apply_registrations(7, &mut p);
    assert_eq!(
        p.calls,
        [
            "blocked",
            "ticker",
            "release-ticker",
            "tocker",
            "release-tocker"
        ]
    );
    let mut p = Probe {
        blocked: true,
        ..Probe::default()
    };
    apply_registrations(7, &mut p);
    assert_eq!(p.calls, ["blocked", "release-both"]);
    assert_eq!(p.applied, [RegistrationAction::ReleaseTickTock { id: 7 }]);
}

#[test]
fn solar_tocker_gate_is_read_after_ticker_effects() {
    for (before, after) in [(0, 1), (255, 0)] {
        let mut p = Probe {
            ticker: true,
            interval: 5,
            parents: [Some(SolarSystem {
                id: 9,
                ticking_contents: true,
                field_3a9: before,
            })]
            .into(),
            field_after_ticker: Some(after),
            ..Probe::default()
        };
        apply_registrations(7, &mut p);
        assert_eq!(
            p.applied[1],
            if after == 0 {
                RegistrationAction::UpdateOrAddTocker {
                    id: 7,
                    atom_type: 0x17,
                    solar_id: 9,
                    raw_interval: 5,
                }
            } else {
                RegistrationAction::ReleaseTocker { id: 7 }
            }
        );
    }
}

#[test]
fn blocking_and_zero_interval_skip_unneeded_getters() {
    let mut p = Probe {
        blocked: true,
        ..Probe::default()
    };
    assert_eq!(
        registration_actions(7, &mut p),
        vec![RegistrationAction::ReleaseTickTock { id: 7 }]
    );
    assert_eq!(p.calls, ["blocked"]);
    let mut p = Probe::default();
    assert_eq!(
        registration_actions(7, &mut p),
        vec![
            RegistrationAction::ReleaseTicker { id: 7 },
            RegistrationAction::ReleaseTocker { id: 7 }
        ]
    );
    assert_eq!(p.calls, ["blocked", "ticker", "tocker"]);
}

#[test]
fn ticker_and_tocker_gates_are_independent_and_nonzero_bits_are_preserved() {
    for ticker in [false, true] {
        for ticking_contents in [false, true] {
            for field_3a9 in [0, 1, 255] {
                for interval in [0, 1, 0x80000000, u32::MAX] {
                    let mut p = Probe {
                        ticker,
                        interval,
                        parents: [Some(SolarSystem {
                            id: 99,
                            ticking_contents,
                            field_3a9,
                        })]
                        .into(),
                        ..Probe::default()
                    };
                    let result = registration_actions(7, &mut p);
                    assert_eq!(
                        result[0],
                        if ticker && ticking_contents {
                            RegistrationAction::UpdateOrAddTicker {
                                id: 7,
                                atom_type: 0x16,
                                solar_id: 99,
                            }
                        } else {
                            RegistrationAction::ReleaseTicker { id: 7 }
                        }
                    );
                    assert_eq!(
                        result[1],
                        if interval != 0 && field_3a9 == 0 {
                            RegistrationAction::UpdateOrAddTocker {
                                id: 7,
                                atom_type: 0x16,
                                solar_id: 99,
                                raw_interval: interval,
                            }
                        } else {
                            RegistrationAction::ReleaseTocker { id: 7 }
                        }
                    );
                    assert_eq!(
                        p.calls.iter().filter(|&&call| call == "parent").count(),
                        usize::from(ticker || interval != 0)
                    );
                }
            }
        }
    }
}

#[test]
fn missing_ticker_parent_is_retried_and_request_types_are_read_separately() {
    let system = SolarSystem {
        id: 9,
        ticking_contents: true,
        field_3a9: 0,
    };
    let mut p = Probe {
        ticker: true,
        interval: 123,
        parents: [None, Some(system)].into(),
        ..Probe::default()
    };
    assert_eq!(
        registration_actions(7, &mut p),
        vec![
            RegistrationAction::ReleaseTicker { id: 7 },
            RegistrationAction::UpdateOrAddTocker {
                id: 7,
                atom_type: 0x16,
                solar_id: 9,
                raw_interval: 123
            }
        ]
    );
    assert_eq!(
        p.calls,
        ["blocked", "ticker", "parent", "tocker", "parent", "type"]
    );
    let mut p = Probe {
        ticker: true,
        interval: 123,
        parents: [Some(system)].into(),
        ..Probe::default()
    };
    registration_actions(7, &mut p);
    assert_eq!(
        p.calls,
        ["blocked", "ticker", "parent", "type", "tocker", "type"]
    );
    let mut p = Probe {
        ticker: true,
        interval: 123,
        parents: [None, None].into(),
        ..Probe::default()
    };
    assert_eq!(
        registration_actions(7, &mut p)[1],
        RegistrationAction::ReleaseTocker { id: 7 }
    );
    assert!(!p.calls.contains(&"type"));
}
