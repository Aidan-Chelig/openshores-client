use super::*;

fn body() -> NativeDpInitialization {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[12] = 12;
    // Unsegmented anatomy gives coincident attachments for all three pairs.
    let mut body = NativeDpInitialization::begin(&dna);
    for part in &mut body.parts {
        part.calculate(&body.dimensions, &mut || panic!()).unwrap();
    }
    body
}

fn splay(part: &NativeDpPartState) -> f32 {
    let NativeDpPartJoints::Leg { splay_48, .. } = part.joints else {
        panic!()
    };
    splay_48
}

#[test]
fn spread_counts_all_coincident_pairs_before_filtering_and_preserves_skipped_state() {
    for selector in [-1, 0, 1, 2, 3] {
        let mut body = body();
        let groups = connection_groups(&body.parts).unwrap();
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|(_, count)| *count == 3));
        apply_spread(
            &mut body.parts,
            &body.dimensions,
            &groups,
            0.2,
            selector,
            1.,
            0.1,
        )
        .unwrap();
        let step = f32::from_bits(0x40490fdb) / 4.;
        for side in [
            NativeDpPartKind::LeftLegAndFoot,
            NativeDpPartKind::RightLegAndFoot,
        ] {
            let mut n = 0;
            for part in body
                .parts
                .iter()
                .rev()
                .filter(|p| p.allocation.kind == side)
            {
                if selected(part, selector) {
                    n += 1;
                    assert_eq!(splay(part), step * n as f32);
                } else {
                    assert_eq!(splay(part), 0.);
                }
            }
        }
        // A subsequent filtered update must not clear the other pair indices.
        let before = body.parts.clone();
        apply_spread(&mut body.parts, &body.dimensions, &groups, 0., 1, 1., 0.1).unwrap();
        for (a, b) in body.parts.iter().zip(before) {
            if is_leg(a) && a.allocation.index != 0 {
                assert_eq!(a.joints, b.joints);
            }
        }
    }
}

#[test]
fn ordinary_triangle_uses_requested_delta_and_zero_uses_group_count() {
    let mut body = body();
    body.dna.0[12] = 4;
    body.dna.0[16] = 1;
    body.dna.0[10] = 0x80;
    body = NativeDpInitialization::begin(&body.dna);
    for part in &mut body.parts {
        part.calculate(&body.dimensions, &mut || panic!()).unwrap();
    }
    let groups = connection_groups(&body.parts).unwrap();
    let d = body.dimensions.fields_4080_through_40b8();
    let height = d[7].max(d[0]);
    apply_spread(
        &mut body.parts,
        &body.dimensions,
        &groups,
        0.2,
        2,
        height,
        0.1,
    )
    .unwrap();
    for part in body.parts.iter().filter(|p| is_leg(p)) {
        assert_eq!(splay(part), 0.2);
    }
    apply_spread(
        &mut body.parts,
        &body.dimensions,
        &groups,
        -0.,
        2,
        height,
        0.1,
    )
    .unwrap();
    for part in body.parts.iter().filter(|p| is_leg(p)) {
        assert_eq!(splay(part), f32::from_bits(0x40490fdb) / 2.);
    }
}

#[test]
fn grouping_is_exact_and_stops_at_first_nonleg_in_reverse_suffix() {
    let mut body = body();
    let last = body.parts.len() - 1;
    body.parts[last].connect_30[2] = f64::EPSILON;
    assert_eq!(connection_groups(&body.parts).unwrap().len(), 3);
    body.parts[last - 1].allocation.kind = NativeDpPartKind::Torso;
    assert_eq!(
        connection_groups(&body.parts).unwrap(),
        vec![(body.parts[last].connect_30, 1)]
    );
    body.parts[last].connect_30[0] = f64::NAN;
    assert_eq!(
        connection_groups(&body.parts).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
}
