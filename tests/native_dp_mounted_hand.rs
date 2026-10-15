//! Opt-in real user-installation check, entirely offline and read-only.
use openshores_client::{
    asset_mount::AssetMount,
    world::{native_body::NativeDpHandResource, native_dna::NativeDhDna},
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn native_hand_selection_reads_and_decodes_user_mounted_resources() {
    let path = std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path");
    let mount = AssetMount::open(path).unwrap();
    let mut decoded = 0;
    let mut missing = 0;
    let mut vertices = 0;
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
    eprintln!(
        "Mounted native hands: {decoded} decoded, {missing} absent, {vertices} raw vertices; PA hand decoded. No native HandLength/render parity claim."
    );
}
