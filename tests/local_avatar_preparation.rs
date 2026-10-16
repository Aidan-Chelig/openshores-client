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
}
