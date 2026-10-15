//! Actual user-supplied body-group records, entirely offline and read-only.
use openshores_client::{
    asset_mount::AssetMount,
    model::{
        ModelVariant, NativeBodyDeformation, NativeBodyModelGeometry, NativeBodyObjectGroups,
        NativeBodyObjectScale, NativeBodyVertexGroup,
    },
    world::{
        native_body::{
            NativeDpHandCache, NativeDpHandResource, NativeDpInitialization, NativeDpPartKind,
        },
        native_dna::NativeDhDna,
    },
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
    let mut deformed_meshes = 0;
    let mut unsupported_meshes = 0;
    let mut changed_points = 0;
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
            // Explicit non-unit fixture values, NOT recovered Person model
            // definitions. This checks real asset composition without claiming
            // native execution, complete model construction or visual parity.
            let mut deformation = NativeBodyDeformation {
                base: definition,
                field_170: 1.2,
                field_178: 1.3,
                field_180: 1.4,
                field_188: 1.5,
                field_190: 1.6,
                fields_25c: [0.; 3],
                fields_228: [0.; 3],
                fields_2c4: [0.; 3],
            };
            NativeBodyModelGeometry {
                fields_00: [1., 0.75, 0., 0., 0., 0.],
                fields_40: [0.; 3],
                fields_58: [1.3, 1.2, 1.5, 1.4, 1.6],
            }
            .apply_to_deformation(&mut deformation);
            match mesh.body_deformed_positions(ModelVariant::N, 1., &deformation, [0.; 3], [0.; 3])
            {
                Ok(points) => {
                    deformed_meshes += 1;
                    assert_eq!(points.len(), mesh.vertices.len());
                    let mut base = definition;
                    if let Some(connection) = state.control_point(0x1cc) {
                        base.fields_1f4 = connection;
                    }
                    for (point, loaded) in points
                        .into_iter()
                        .zip(mesh.loaded_positions(ModelVariant::N, 1.).unwrap())
                    {
                        assert!(point.iter().all(|v| v.is_finite()));
                        changed_points +=
                            usize::from(point != base.apply_unweighted_vertex(loaded));
                    }
                }
                Err(error) => {
                    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
                    unsupported_meshes += 1;
                }
            }
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
    assert!(deformed_meshes > 0 && changed_points > 0);
    eprintln!(
        "Mounted body groups: {groups} parsed, {keys} native position keys, {pivots} nonzero pivots; {retained} retained groups, {connections} connection writes, {point_only_meshes} point-only meshes. No weighted-deformation or rendered-avatar parity claim."
    );
    eprintln!(
        "Explicit-definition mesh deformation: {deformed_meshes} supported, {unsupported_meshes} unsupported, {changed_points} points differ from the unweighted branch. Not complete avatar generation or rendering."
    );
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn initialized_part_skin_requests_load_and_deform_mounted_models() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[12] = 4;
    dna.0[14] = 0x50;
    dna.0[20] = 0x40;
    let mut body = NativeDpInitialization::begin(&dna);
    body.finish_part_calculations(0, &mut NativeDpHandCache::new(&mount))
        .unwrap();
    let mut resources = 0;
    let mut points = 0;
    for (index, part) in body.parts().iter().enumerate() {
        let requests: Vec<_> = match part.allocation.kind {
            NativeDpPartKind::LeftArmAndHand | NativeDpPartKind::RightArmAndHand => body
                .arm_skin_geometry_requests(index, 0, [1.; 4])
                .unwrap()
                .into_iter()
                .map(|r| (r.resource, r.geometry))
                .collect(),
            NativeDpPartKind::LeftLegAndFoot | NativeDpPartKind::RightLegAndFoot => body
                .leg_skin_geometry_requests(index, 0, [1.; 4])
                .unwrap()
                .into_iter()
                .map(|r| (r.resource, r.geometry))
                .collect(),
            NativeDpPartKind::Tail => body
                .tail_skin_geometry_request(index, [1.; 4])
                .unwrap()
                .into_iter()
                .map(|r| (r.resource, r.geometry))
                .collect(),
            NativeDpPartKind::Torso => body
                .torso_skin_geometry_plan(index, 0, [1.; 4])
                .unwrap()
                .into_iter()
                .flat_map(|p| p.fallback)
                .map(|r| (r.resource, r.geometry))
                .collect(),
            _ => continue,
        };
        for (resource, geometry) in requests {
            let model = openshores_client::model::Model::decode(
                &mount.read_resource(&resource).unwrap(),
                ModelVariant::N,
            )
            .unwrap();
            resources += 1;
            for mesh in model.meshes {
                // Explicit metadata fixture defaults: not a reconstructed
                // object constructor, device object or rendered model.
                let mut definition = NativeBodyDeformation {
                    base: NativeBodyObjectScale {
                        field_140: 1.,
                        fields_1c0: [0.; 3],
                        fields_1f4: [0.; 3],
                    },
                    field_170: 1.,
                    field_178: 1.,
                    field_180: 1.,
                    field_188: 1.,
                    field_190: 1.,
                    fields_228: [0.; 3],
                    fields_2c4: [0.; 3],
                    fields_25c: [0.; 3],
                };
                geometry.apply_to_deformation(&mut definition);
                let output = mesh
                    .body_deformed_positions(ModelVariant::N, 1., &definition, [0.; 3], [0.; 3])
                    .unwrap();
                points += output.len();
                assert_eq!(output.len(), mesh.vertices.len());
                assert!(output.iter().flatten().all(|value| value.is_finite()));
            }
        }
    }
    assert_eq!(resources, 11);
    assert!(points > 0);
    eprintln!(
        "Initialized part skin geometry: {resources} mounted resources, {points} CPU vertices; explicit metadata fixture defaults, no complete model/rendering parity claim."
    );
}
