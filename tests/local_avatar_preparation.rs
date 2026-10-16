//! Source-profile receive -> normal body preparation. No original runtime/server.
use openshores_client::{
    asset_mount::AssetMount,
    client_model::{AvatarContentKey, ClientModel, PreparedNormalAvatar},
    client_runtime::Event,
    local_scene::Bootstrap,
    protocol::QtWriter,
    scene::InitSucceeded,
};

fn bootstrap(dna: &[u8]) -> Event {
    let mut body = QtWriter::default();
    body.write_u8(0); // omitted name/empire
    body.write_u8(1); // creature DNA update
    body.write_u8(0x24); // opaque source pose byte, not a DNA discriminator
    body.write_bytes(Some(dna)).unwrap();
    for _ in 0..3 {
        body.write_u8(0);
    } // stamina/growth/person flags
    let mut person = vec![0x12];
    person.extend(42u32.to_be_bytes());
    person.extend(0i64.to_be_bytes());
    person.push(0);
    person.extend(body.into_bytes());
    let mut active = vec![0x2a];
    active.extend(42u32.to_be_bytes());
    Event::Bootstrap(Box::new(Bootstrap {
        avatar_id: 42,
        initialization: InitSucceeded {
            motd: None,
            chat_port: 1,
            autime_usec: 0,
        },
        manifest: vec![42],
        time_low: 0,
        packet_counts: Default::default(),
        bytes_received: 0,
        packets: vec![person, active],
    }))
}

#[test]
fn repeated_bootstrap_changes_ownership_even_for_the_same_player_id() {
    let mut model = ClientModel::default();
    model.apply(bootstrap(&[1; 24])).unwrap();
    assert_eq!(model.session_generation, 1);
    assert_eq!(model.local_player_dna().unwrap().unwrap().received, [1; 24]);
    model.apply(bootstrap(&[2; 24])).unwrap();
    assert_eq!(model.session_generation, 2);
    assert_eq!(model.local_player_dna().unwrap().unwrap().received, [2; 24]);
    let before = model.session_generation;
    model
        .apply(Event::SentScene(
            openshores_client::commands::SceneCommand::PositionReport {
                projected: [1., 2., 3.],
                rotation: [0., 0., 1.],
            },
        ))
        .unwrap();
    assert_eq!(model.session_generation, before);
    model.session_generation = u64::MAX;
    assert!(model.apply(bootstrap(&[3; 24])).is_err());
    assert_eq!(model.local_player_dna().unwrap().unwrap().received, [2; 24]);
}

#[test]
#[ignore = "requires mounted installation and OPENSHORES_TEST_PLAYER_DNA_HEX from the supplied server"]
fn mounted_received_server_dna_prepares_without_a_preview_fixture_substitution() {
    let hex = std::env::var("OPENSHORES_TEST_PLAYER_DNA_HEX").expect("explicit server DNA");
    assert_eq!(hex.len(), 48);
    let bytes: Vec<_> = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation"),
    )
    .unwrap();
    let mut model = ClientModel::default();
    model.apply(bootstrap(&bytes)).unwrap();
    let received = model.local_player_dna().unwrap().unwrap();
    assert_eq!(received.received.as_slice(), bytes);
    let key = AvatarContentKey::new(received.model.clone(), 1, 60., true).unwrap();
    let prepared = PreparedNormalAvatar::generate(&mount, &key).unwrap();
    assert_eq!(prepared.native.dna, received.model);
    // Repair topology and normals are retained before posing. Sampling another
    // skeleton must not rebuild repairs, change materials or accumulate motion.
    let mut moved = prepared.native.body.bones().clone();
    for bone in &mut moved.slots {
        bone.matrix_00[12] += 0.1;
    }
    assert_eq!(prepared.animation_surfaces.len(), prepared.surfaces.len());
    for (batch, standing) in prepared.animation_surfaces.iter().zip(&prepared.surfaces) {
        let before = batch.skin(prepared.native.body.bones()).unwrap();
        assert_eq!(before.corners, standing.corners);
        let shifted = batch.skin(&moved).unwrap();
        assert_eq!(shifted.surface_index, standing.surface_index);
        assert_eq!(shifted.part_index, standing.part_index);
        assert_eq!(shifted.material_memberships, standing.material_memberships);
        for (a, b) in before.corners.iter().zip(&shifted.corners) {
            assert_eq!(a.uv, b.uv);
            assert_eq!(a.top_face, b.top_face);
            assert_eq!(a.source_vertex, b.source_vertex);
            assert_eq!(a.normal, b.normal);
            let shift = b.position[0] / b.position[3] - a.position[0] / a.position[3];
            assert!((shift - 0.1).abs() < 1e-5);
        }
        assert_eq!(
            batch.skin(prepared.native.body.bones()).unwrap().corners,
            before.corners
        );
    }
    {
        use openshores_client::world::{
            native_body::{NativeDpFootCache, NativeDpGait, NativeDpHandCache},
            native_creature::NativePoseRhythmTimer,
        };
        struct Timer(i64);
        impl NativePoseRhythmTimer for Timer {
            fn start(&mut self) -> std::io::Result<()> {
                Ok(())
            }
            fn msecs_since_reference(&mut self) -> std::io::Result<i64> {
                Ok(self.0)
            }
        }
        let mut body = prepared.native.body.clone();
        let mut hands = NativeDpHandCache::new(&mount);
        let mut feet = NativeDpFootCache::new(&mount);
        for gait in [NativeDpGait::Walk, NativeDpGait::Run] {
            let mut samples = Vec::new();
            for millis in [0, 125, 0] {
                let sample = body
                    .pose_locomotion(gait, 4., 0, &mut hands, &mut feet, &mut Timer(millis))
                    .unwrap();
                assert!(sample.height.is_finite() && sample.torso_angle.is_finite());
                assert_eq!(
                    sample.pose.fields_3c,
                    [if gait == NativeDpGait::Walk {
                        0x26
                    } else {
                        0x25
                    }; 10]
                );
                for batch in &prepared.animation_surfaces {
                    batch.skin(body.bones()).unwrap();
                }
                samples.push(body.bones().clone());
            }
            assert_eq!(
                samples[0], samples[2],
                "full-body samples must reset, not accumulate"
            );
            assert_ne!(samples[0], samples[1]);
        }
        body.pose_neutral_standing(0, &mut hands, &mut feet)
            .unwrap();
        assert_eq!(
            body.bones(),
            prepared.native.body.bones(),
            "stopping must restore original Stand"
        );
    }
    let ground_height = prepared.native.standing_ground_clearance(&mount).unwrap();
    let camera = prepared.native.standing_camera_location(false).unwrap();
    println!("Received native Stand camera location: {camera:?}");
    assert!(camera.iter().all(|value| value.is_finite()));
    let low = prepared
        .surfaces
        .iter()
        .flat_map(|s| &s.corners)
        .map(|c| c.position[2] / c.position[3])
        .fold(f32::INFINITY, f32::min);
    let high = prepared
        .surfaces
        .iter()
        .flat_map(|s| &s.corners)
        .map(|c| c.position[2] / c.position[3])
        .fold(f32::NEG_INFINITY, f32::max);
    println!(
        "Received avatar anchor: pose height={}, ground CG height={ground_height}, drawn Z=[{low},{high}], sole residual={}",
        prepared.native.standing.height,
        low + ground_height
    );
    assert!(ground_height.is_finite() && ground_height > 0.);
    assert!(low < 0. && high > 0., "origin is not a soles origin");
    assert!(ground_height >= prepared.native.standing.height);
    println!("AVATAR_ANCHOR_METRICS {ground_height} {low} {high}");
    assert!(!prepared.surfaces.is_empty());
    assert!(!prepared.atlas_png.is_empty());
    assert!(
        prepared
            .surfaces
            .iter()
            .all(|s| !s.corners.is_empty() && s.corners.len() == s.material_memberships.len() * 3)
    );
    println!(
        "Received server appearance: {} native parts, {} posed render batches",
        prepared.native.body.parts().len(),
        prepared.surfaces.len()
    );
    let mut prepared = prepared;
    prepared.native.standing.pose.fields_3c[0] = 0;
    assert!(prepared.native.standing_camera_location(false).is_err());
    assert!(prepared.native.standing_ground_clearance(&mount).is_err());
    prepared.native.standing.pose.fields_3c[0] = 0x24;
    let mut animation = prepared.into_animation(&mount).unwrap();
    // Cached-only samplers no longer retain mount access, including zero-offset
    // sentinel cases. Every runtime sample here must succeed without asset I/O.
    let stand = animation.sample(None, 0., 0).unwrap();
    assert_eq!(stand.eye, camera);
    for gait in [
        openshores_client::world::native_body::NativeDpGait::Walk,
        openshores_client::world::native_body::NativeDpGait::Run,
    ] {
        let a = animation.sample(Some(gait), 4., 0).unwrap();
        let b = animation.sample(Some(gait), 4., 125).unwrap();
        let repeat = animation.sample(Some(gait), 4., 0).unwrap();
        assert_eq!(a.eye, repeat.eye);
        assert!(b.eye.iter().all(|v| v.is_finite()));
        assert!(
            a.surfaces
                .iter()
                .zip(&b.surfaces)
                .any(|(a, b)| a.corners != b.corners)
        );
        for (a, b) in a.surfaces.iter().zip(&repeat.surfaces) {
            assert_eq!(a.corners, b.corners);
        }
    }
    let mut mixed = openshores_client::world::native_creature::NativeDpBodyPoseState {
        fields_3c: [0x24; 10],
        field_6c: false,
    };
    mixed.fields_3c[0] = 0x26;
    mixed.fields_3c[2] = 0x25;
    mixed.fields_3c[4] = 0x25;
    let a = animation.sample_channels(&mixed, 4., 125).unwrap();
    let mut bad = mixed.clone();
    bad.fields_3c[2] = 0x12;
    assert!(animation.sample_channels(&bad, 4., 125).is_err());
    let repeat = animation.sample_channels(&mixed, 4., 125).unwrap();
    assert_eq!(a.eye, repeat.eye);
    for (a, b) in a.surfaces.iter().zip(&repeat.surfaces) {
        assert_eq!(a.corners, b.corners);
    }
    let stopped = animation.sample(None, 0., 500).unwrap();
    assert_eq!(stopped.eye, stand.eye);
    for (a, b) in stand.surfaces.iter().zip(&stopped.surfaces) {
        assert_eq!(a.corners, b.corners);
    }
    let neutral = openshores_client::world::native_creature::NativeDpBodyPoseState {
        fields_3c: [0x24; 10],
        field_6c: false,
    };
    let received_stand = animation.sample_channels(&neutral, 0., 500).unwrap();
    assert_eq!(received_stand.eye, stand.eye);
    for (a, b) in stand.surfaces.iter().zip(&received_stand.surfaces) {
        assert_eq!(a.corners, b.corners);
    }
}
