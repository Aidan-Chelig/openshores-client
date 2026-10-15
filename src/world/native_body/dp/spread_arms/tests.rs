use super::*;

fn body() -> NativeDpInitialization {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 3;
    let mut body = NativeDpInitialization::begin(&dna);
    for part in &mut body.parts {
        part.calculate(&body.dimensions, &mut || Ok(0.5)).unwrap();
    }
    body
}
fn spread(part: &NativeDpPartState) -> (f32, f32) {
    let NativeDpPartJoints::Arm {
        lift_48, splay_4c, ..
    } = part.joints
    else {
        panic!()
    };
    (lift_48, splay_4c)
}

#[test]
fn forward_spacing_accumulates_per_exact_connection_and_selects_sides_not_pairs() {
    for selector in [-1, 0, 1, 2, 3] {
        let mut body = body();
        // Force all sides into one exact group to prove excluded members
        // consume spacing increments, unlike the leg spreading routine.
        for part in &mut body.parts {
            if ordinary_arm(part) {
                part.connect_30 = [0.; 3];
            }
        }
        let before = body.bones.clone();
        body.spread_arms(0.25, 0.5, selector).unwrap();
        let mut n = 0;
        for part in body.parts.iter().filter(|p| ordinary_arm(p)) {
            n += 1;
            assert_eq!(
                spread(part),
                if selected(part, selector) {
                    (0.375 + 0.125 * n as f32, 0.25 * n as f32)
                } else {
                    (0., 0.)
                }
            );
        }
        assert_eq!(n, 6);
        assert_eq!(before, body.bones);
    }
}

#[test]
fn skipped_wings_and_unselected_state_remain_untouched() {
    let mut body = body();
    body.spread_arms(0.25, 0.5, 2).unwrap();
    let before = body.parts.clone();
    body.spread_arms(0.5, 0.25, 0).unwrap();
    for (part, old) in body.parts.iter().zip(before) {
        if part.allocation.kind == NativeDpPartKind::RightArmAndHand {
            assert_eq!(part.joints, old.joints);
        }
    }
    for part in &mut body.parts {
        if ordinary_arm(part) {
            part.allocation.dna.0[2] = 2;
        }
    }
    let before = body.parts.clone();
    body.spread_arms(0.75, 1., 2).unwrap();
    assert_eq!(body.parts, before);
}

#[test]
fn map_rejects_nonfinite_keys_but_does_not_epsilon_merge_attachments() {
    let mut body = body();
    let arms: Vec<_> = body.parts.iter_mut().filter(|p| ordinary_arm(p)).collect();
    for (i, part) in arms.into_iter().enumerate() {
        part.connect_30 = [0., 0., i as f64 * f64::EPSILON];
    }
    body.spread_arms(0.25, 0.5, 2).unwrap();
    for part in body.parts.iter().filter(|p| ordinary_arm(p)) {
        assert_eq!(spread(part), (0.5, 0.25));
    }
    body.parts.last_mut().unwrap().connect_30[0] = f64::NAN;
    assert_eq!(
        body.spread_arms(0.25, 0.5, 2).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn one_time_calculations_preserve_native_lift_and_splay_fields() {
    let mut body = body();
    body.spread_arms(0.25, 0.5, 2).unwrap();
    let expected: Vec<_> = body
        .parts
        .iter()
        .filter(|p| ordinary_arm(p))
        .map(spread)
        .collect();
    for part in &mut body.parts {
        part.calculate(&body.dimensions, &mut || Ok(0.5)).unwrap();
    }
    let actual: Vec<_> = body
        .parts
        .iter()
        .filter(|p| ordinary_arm(p))
        .map(spread)
        .collect();
    assert_eq!(actual, expected);
}
