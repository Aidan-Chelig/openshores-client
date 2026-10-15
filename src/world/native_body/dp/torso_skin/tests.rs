use super::*;

fn part(dna: NativeDhDna) -> NativeDpPartState {
    NativeDpPartState {
        allocation: NativeDpAllocatedPart {
            kind: NativeDpPartKind::Torso,
            index: 0,
            dna,
        },
        connect_30: [999.; 3],
        joints: NativeDpPartJoints::None,
    }
}

#[test]
fn native_combined_key_and_ordered_fallback_keep_bones_and_translation() {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[20] = 0x40;
    let dimensions = NativeDpDimensions::from_dna(&dna);
    let p = part(dna.clone());
    let plan = p
        .torso_skin_plan(&dna, &dimensions, 0, [1., 2., 3., 4.])
        .unwrap()
        .unwrap();
    assert_eq!(plan.combined_key, ":data/0528");
    assert_eq!(plan.bones, [0, 0, 1]);
    assert_eq!(
        plan.fallback
            .iter()
            .map(|p| p.resource.as_str())
            .collect::<Vec<_>>(),
        [
            ":data/0NoLimbClosedToNoLimb.3dsN",
            ":data/0NoLimbToButt.3dsN"
        ]
    );
    assert_eq!(plan.fallback[0].bones, [0, 0, 1]);
    assert_eq!(plan.fallback[1].bones, [0, 1, 1]);
    let step = f64::from(dimensions.fields_4080_through_40b8()[10]);
    assert_eq!(plan.geometry.fields_00[0], step);
    assert_eq!(plan.geometry.fields_40, [0., -step, 0.]);
    assert_eq!(plan.fallback[1].geometry.fields_40, [0., -step - step, 0.]);
}

#[test]
fn crest_pieces_follow_each_segment_and_pa_suppresses_all_crests() {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[20] = 0x40;
    dna.0[18] = 0x20;
    dna.0[4] = 0x0c;
    let dimensions = NativeDpDimensions::from_dna(&dna);
    let p = part(dna.clone());
    let plan = p
        .torso_skin_plan(&dna, &dimensions, 0, [0.; 4])
        .unwrap()
        .unwrap();
    assert_eq!(plan.combined_key, ":data/05TC3228TC32");
    assert_eq!(plan.fallback.len(), 4);
    for pair in plan.fallback.chunks_exact(2) {
        assert!(!pair[0].crest && pair[1].crest);
        assert_eq!(pair[1].resource, ":data/0TorsoCrest32.3dsN");
        assert_eq!(pair[0].bones, pair[1].bones);
        assert_eq!(pair[0].geometry, pair[1].geometry);
    }
    let pa = p
        .torso_skin_plan(&dna, &dimensions, 0x41, [0.; 4])
        .unwrap()
        .unwrap();
    assert_eq!(pa.combined_key, ":data/PA528");
    assert_eq!(pa.fallback.len(), 2);
    assert!(pa.fallback.iter().all(|p| !p.crest));
}

#[test]
fn zero_segments_none_wrong_kind_and_incomplete_owner_errors() {
    let dna = NativeDhDna([0; 24]);
    let dimensions = NativeDpDimensions::from_dna(&dna);
    let mut p = part(dna.clone());
    assert!(
        p.torso_skin_plan(&dna, &dimensions, 0, [0.; 4])
            .unwrap()
            .is_none()
    );
    p.allocation.kind = NativeDpPartKind::Tail;
    assert_eq!(
        p.torso_skin_plan(&dna, &dimensions, 0, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(
        NativeDpInitialization::begin(&dna)
            .torso_skin_geometry_plan(0, 0, [0.; 4])
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
}
