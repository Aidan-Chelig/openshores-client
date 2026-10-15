//! Opt-in real user-installation check, entirely offline and read-only.
use openshores_client::{
    asset_mount::AssetMount,
    world::{
        native_body::{
            NativeDpDimensions, NativeDpFootCache, NativeDpFootResource, NativeDpHandCache,
            NativeDpHandResource, NativeDpInitialization, NativeDpPartJoints,
        },
        native_dna::NativeDhDna,
    },
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn native_hand_selection_reads_and_decodes_user_mounted_resources() {
    let path = std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path");
    let mount = AssetMount::open(path).unwrap();
    let mut decoded = 0;
    let mut missing = 0;
    let mut vertices = 0;
    let mut cache = NativeDpHandCache::new(&mount);
    for prefix in 0..8 {
        for hand in 0..8 {
            let mut dna = NativeDhDna([0; 24]);
            dna.0[16] = prefix;
            dna.0[11] = hand << 5;
            let request = NativeDpHandResource::new(&dna, 0, false);
            match request.read_model(&mount) {
                Ok(model) => {
                    assert!(!model.source.is_empty());
                    vertices += model
                        .meshes
                        .iter()
                        .map(|mesh| mesh.vertices.len())
                        .sum::<usize>();
                    decoded += 1;
                    let length = cache.hand_length(&dna, 0, false).unwrap();
                    assert!(
                        length.is_finite() && length >= 0.0,
                        "{}: {length}",
                        request.resource()
                    );
                    assert_eq!(
                        cache.hand_length(&dna, 0, false).unwrap().to_bits(),
                        length.to_bits()
                    );
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    missing += 1;
                }
                Err(error) => panic!("{}: {error}", request.resource()),
            }
        }
    }
    assert!(
        decoded > 0 && vertices > 0,
        "installation has no selected hand geometry"
    );
    let pa = NativeDpHandResource::new(&NativeDhDna([0; 24]), 0x41, false);
    let model = pa.read_model(&mount).expect("actual PA hand resource");
    assert!(!model.meshes.is_empty());
    let pa_length = cache
        .hand_length(&NativeDhDna([0; 24]), 0x41, false)
        .unwrap();
    assert!(pa_length.is_finite() && pa_length >= 0.0);
    eprintln!(
        "Mounted native hands: {decoded} decoded, {missing} absent, {vertices} raw vertices; PA hand decoded. No native HandLength/render parity claim."
    );
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn native_foot_measurements_and_standing_height_use_mounted_resources() {
    let path = std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path");
    let mount = AssetMount::open(path).unwrap();
    let mut cache = NativeDpFootCache::new(&mount);
    let mut decoded = 0;
    for field in [0, 0x41] {
        for prefix in 0..8 {
            for foot in 0..8 {
                let mut dna = NativeDhDna([0; 24]);
                dna.0[16] = prefix;
                dna.0[10] = foot << 5;
                dna.0[12] = 4;
                let request = NativeDpFootResource::new(&dna, field, false);
                let model = request
                    .read_model(&mount)
                    .unwrap_or_else(|e| panic!("{}: {e}", request.resource()));
                assert!(!model.meshes.is_empty());
                decoded += 1;
                let length = cache.foot_length(&dna, field, false).unwrap();
                let height = cache.foot_height(&dna, field).unwrap();
                assert!(
                    length.is_finite() && height.is_finite(),
                    "{}: {length}, {height}",
                    request.resource()
                );
                assert_eq!(
                    cache.foot_length(&dna, field, false).unwrap().to_bits(),
                    length.to_bits()
                );
                assert_eq!(
                    cache.foot_height(&dna, field).unwrap().to_bits(),
                    height.to_bits()
                );
                for class in 0..4 {
                    dna.0[1] = class << 2;
                    let dimensions = NativeDpDimensions::from_dna(&dna);
                    for include in [false, true] {
                        assert!(
                            cache
                                .standing_height(&dna, &dimensions, field, include)
                                .unwrap()
                                .is_finite()
                        );
                        let mut pose =
                            openshores_client::world::native_creature::NativeDpBodyPoseState {
                                fields_3c: [80; 10],
                                ..Default::default()
                            };
                        // Actual body field48 is the fourth word; other words
                        // deliberately differ so a mistaken selector is caught.
                        for code in 0..=255 {
                            pose.fields_3c[3] = code;
                            let value = cache
                                .pose_height(&dna, &dimensions, field, &pose, include)
                                .unwrap();
                            assert!(value.is_finite(), "pose {code}, {}", request.resource());
                            assert_eq!(
                                cache
                                    .pose_height(&dna, &dimensions, field, &pose, include)
                                    .unwrap()
                                    .to_bits(),
                                value.to_bits()
                            );
                        }
                    }
                }
            }
        }
    }
    eprintln!(
        "Mounted foot requests decoded: {decoded}; numbered and both PA ankle resources, cached measurements and standing heights verified in Rust. No native runtime/render parity claim."
    );
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn native_part_initialization_consumes_mounted_body_hand_geometry() {
    let path = std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path");
    let mount = AssetMount::open(path).unwrap();
    let mut cache = NativeDpHandCache::new(&mount);
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 3;
    dna.0[2] = 2;
    dna.0[5] = 2;
    dna.0[12] = 12;
    dna.0[14] = 0x30;
    dna.0[20] = 0x60;
    let hand = cache.hand_length(&dna, 0, false).unwrap();
    let mut body = NativeDpInitialization::begin(&dna);
    assert_eq!(body.parts().len(), 16);
    assert!(!body.parts_calculated());
    body.finish_part_calculations(0, &mut cache).unwrap();
    assert!(body.parts_calculated());
    for part in body.parts() {
        assert!(part.connect_30.iter().all(|n| n.is_finite()));
        let attachment = part.current_connect_loc(body.bones()).unwrap();
        assert!(attachment.iter().all(|n| n.is_finite()));
        if matches!(part.joints, NativeDpPartJoints::Head { .. }) {
            let mut bones = body.bones().clone();
            part.bend_neck(&mut bones, body.dimensions(), 0.125)
                .unwrap();
            part.rotate_neck(&mut bones, 0.25).unwrap();
            part.lean_neck(&mut bones, -0.375).unwrap();
            let moved = part.current_connect_loc(&bones).unwrap();
            for axis in 0..3 {
                assert!((moved[axis] - attachment[axis]).abs() < 1e-12);
            }
        }
        if let NativeDpPartJoints::Arm {
            field_70, field_88, ..
        } = part.joints
        {
            assert_eq!(
                field_88,
                [
                    field_70[0],
                    f64::from(hand) * 0.5 + field_70[1],
                    field_70[2]
                ]
            );
        }
    }
    let completed = body.parts().to_vec();
    body.finish_part_calculations(0, &mut cache).unwrap();
    assert_eq!(body.parts(), completed);
}
