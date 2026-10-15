//! Actual user-supplied body-group records, entirely offline and read-only.
use openshores_client::{
    asset_mount::AssetMount,
    model::{ModelVariant, NativeBodyObjectGroups, NativeBodyObjectScale, NativeBodyVertexGroup},
    world::{native_body::NativeDpHandResource, native_dna::NativeDhDna},
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn mounted_body_groups_preserve_native_membership_and_pivot_separation() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut groups = 0;
    let mut keys = 0;
    let mut pivots = 0;
    let mut retained = 0;
    let mut connections = 0;
    let mut point_only_meshes = 0;
    for selector in 0..=64 {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[16] = selector / 8;
        dna.0[11] = (selector % 8) << 5;
        let resource =
            NativeDpHandResource::new(&dna, if selector == 64 { 0x41 } else { 0 }, false);
        let model = resource.read_model(&mount).unwrap();
        for mesh in model.meshes {
            let state = NativeBodyObjectGroups::from_mesh(&mesh).unwrap();
            retained += state.retained_groups().len();
            connections += usize::from(state.control_point(0x1cc).is_some());
            for raw in &mesh.vertex_groups {
                let parsed = NativeBodyVertexGroup::from_group(raw).unwrap();
                groups += 1;
                pivots += usize::from(parsed.point_28.iter().any(|v| *v != 0.));
                let members = raw.members.as_deref().unwrap_or(&[]);
                // Native single-record groups never insert a QMap weight.
                if members.len() <= 1 {
                    assert_eq!(parsed.entries().count(), 0);
                }
                for (point, weight) in parsed.entries() {
                    keys += 1;
                    assert!(members.iter().any(|member| member.position == point));
                    assert_eq!(
                        parsed.matching_weight(point.map(f64::from)).unwrap(),
                        Some(weight)
                    );
                    assert!(weight.is_finite());
                }
                assert!(parsed.entries().count() <= members.len());
            }
            let definition = NativeBodyObjectScale {
                field_140: 0.75,
                fields_1c0: [0.; 3],
                fields_1f4: [0.; 3],
            };
            if state.retained_groups().is_empty() {
                point_only_meshes += 1;
                let actual = mesh
                    .body_unweighted_positions(ModelVariant::N, 1., &definition)
                    .unwrap();
                let mut expected_definition = definition;
                if let Some(connection) = state.control_point(0x1cc) {
                    expected_definition.fields_1f4 = connection;
                }
                let expected: Vec<_> = mesh
                    .loaded_positions(ModelVariant::N, 1.)
                    .unwrap()
                    .into_iter()
                    .map(|point| expected_definition.apply_unweighted_vertex(point))
                    .collect();
                assert_eq!(actual, expected);
            } else {
                assert_eq!(
                    mesh.body_unweighted_positions(ModelVariant::N, 1., &definition)
                        .unwrap_err()
                        .kind(),
                    std::io::ErrorKind::Unsupported
                );
            }
        }
    }
    assert!(groups > 0 && keys > 0 && pivots > 0);
    eprintln!(
        "Mounted body groups: {groups} parsed, {keys} native position keys, {pivots} nonzero pivots; {retained} retained groups, {connections} connection writes, {point_only_meshes} point-only meshes. No weighted-deformation or rendered-avatar parity claim."
    );
}
