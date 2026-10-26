//! Native side-facing death legs; static anchors in native_dead_side_legs.py.
use super::*;
use std::io;

fn angles(
    dna: &NativeDhDna,
    splay: f32,
    near: bool,
    standing: impl FnOnce() -> io::Result<NativeDpLegStandAngles>,
) -> io::Result<NativeDpLegStandAngles> {
    let (hip, knee, ankle) = if dna.0[12] & 12 == 4 && splay < f32::from_bits(0x3f490fdb) {
        if near {
            (0xc021f770, 0xbf5f66f3, 0x3f32b8c3)
        } else {
            (0xc0437a15, 0xbe32b8c2, 0x3fc90fdb)
        }
    } else {
        let stand = standing()?;
        let (hip, knee) = match (near, stand.hip > 0.) {
            (true, true) => (0x3f9c61aa, 0xc03de44e),
            (true, false) => (0xbfa78d36, 0xbf060a92),
            (false, true) => (0x3fc49809, 0xbd0efa35),
            (false, false) => (0xbfc90fdb, 0xbf060a92),
        };
        return Ok(NativeDpLegStandAngles {
            hip: f32::from_bits(hip),
            knee: f32::from_bits(knee),
            ankle: stand.ankle,
        });
    };
    Ok(NativeDpLegStandAngles {
        hip: f32::from_bits(hip),
        knee: f32::from_bits(knee),
        ankle: f32::from_bits(ankle),
    })
}

impl NativeDpInitialization {
    pub fn dead_legs_facing_left(
        &mut self,
        selector: i32,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
    ) -> io::Result<()> {
        self.dead_legs_facing_side(selector, true, field_4068, feet)
    }

    pub fn dead_legs_facing_right(
        &mut self,
        selector: i32,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
    ) -> io::Result<()> {
        self.dead_legs_facing_side(selector, false, field_4068, feet)
    }

    fn dead_legs_facing_side(
        &mut self,
        selector: i32,
        facing_left: bool,
        field_4068: i32,
        feet: &mut NativeDpFootCache<'_>,
    ) -> io::Result<()> {
        self.spread_legs(f32::from_bits(0xbd0efa35), selector, field_4068, feet)?;
        for part in self.parts.iter().skip(1).rev().take_while(|part| {
            matches!(
                part.allocation.kind,
                NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot
            )
        }) {
            let selected = match selector {
                0 => part.allocation.index != 0,
                1 => part.allocation.index == 0,
                2 => true,
                _ => false,
            };
            if !selected {
                continue;
            }
            let NativeDpPartJoints::Leg { splay_48, .. } = part.joints else {
                return Err(io::Error::other("side-facing dead leg missing joint state"));
            };
            let near = (part.allocation.kind == NativeDpPartKind::LeftLegAndFoot) == facing_left;
            let selected = angles(&part.allocation.dna, splay_48, near, || {
                let height =
                    feet.standing_height(&self.dna, &self.dimensions, field_4068, false)?;
                feet.leg_stand_angles(
                    &self.dna,
                    &part.allocation.dna,
                    &self.dimensions,
                    field_4068,
                    height,
                )
            })?;
            part.stand_leg(&mut self.bones, selected)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn specialized_single_pair_branch_is_signed_strict_and_skips_measurements() {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[12] = 4;
        for (near, hip, knee, ankle) in [
            (true, 0xc021f770, 0xbf5f66f3, 0x3f32b8c3),
            (false, 0xc0437a15, 0xbe32b8c2, 0x3fc90fdb),
        ] {
            let result = angles(&dna, -2., near, || {
                panic!("unnecessary standing measurement")
            })
            .unwrap();
            assert_eq!(
                [
                    result.hip.to_bits(),
                    result.knee.to_bits(),
                    result.ankle.to_bits()
                ],
                [hip, knee, ankle]
            );
        }
        for splay in [f32::from_bits(0x3f490fdb), 1., f32::NAN] {
            assert!(
                angles(&dna, splay, true, || Err(io::Error::other(
                    "standing required"
                )))
                .is_err()
            );
        }
        dna.0[12] = 12;
        assert!(
            angles(&dna, 0., true, || Err(io::Error::other(
                "standing required"
            )))
            .is_err()
        );
    }

    #[test]
    fn general_branch_preserves_solved_ankle_and_uses_ordered_positive_hip_test() {
        for (near, positive, hip, knee) in [
            (true, true, 0x3f9c61aa, 0xc03de44e),
            (true, false, 0xbfa78d36, 0xbf060a92),
            (false, true, 0x3fc49809, 0xbd0efa35),
            (false, false, 0xbfc90fdb, 0xbf060a92),
        ] {
            for input in if positive {
                vec![0.1]
            } else {
                vec![-1., -0., 0., f32::NAN]
            } {
                let result = angles(&NativeDhDna([0; 24]), 0., near, || {
                    Ok(NativeDpLegStandAngles {
                        hip: input,
                        knee: 99.,
                        ankle: 0.7,
                    })
                })
                .unwrap();
                assert_eq!(result.hip.to_bits(), hip);
                assert_eq!(result.knee.to_bits(), knee);
                assert_eq!(result.ankle, 0.7);
            }
        }
    }
}
