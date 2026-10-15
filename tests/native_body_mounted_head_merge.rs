//! Mounted head object geometry integration, not full native model instances.
use openshores_client::{
    asset_mount::AssetMount,
    model::{Model, ModelVariant, NativeBodyFaceMetadata, NativeBodyMergeGeometry},
    world::{
        native_body::{NativeDpHandCache, NativeDpInitialization, NativeDpSkinPlan},
        native_dna::NativeDhDna,
    },
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; entirely offline"]
fn mounted_head_parts_load_and_run_composed_native_geometry_stages() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[12] = 4;
    dna.0[14] = 0x51;
    dna.0[20] = 0x40;
    let mut body = NativeDpInitialization::begin(&dna);
    body.finish_part_calculations(0, &mut NativeDpHandCache::new(&mount))
        .unwrap();
    let plan = body
        .skin_geometry_plans(0, 0, [1.; 4])
        .unwrap()
        .into_iter()
        .find_map(|part| match part.plan {
            NativeDpSkinPlan::Head(plan) => Some(plan),
            _ => None,
        })
        .expect("head plan");
    let mut destination: Option<NativeBodyMergeGeometry> = None;
    let mut total_faces = 0;
    let mut resources = 0;
    let mut updated_corners = 0;
    let mut incomplete = 0;
    for piece in plan.fallback {
        let model = Model::decode(
            &mount.read_resource(&piece.resource).unwrap(),
            ModelVariant::N,
        )
        .unwrap();
        assert_eq!(
            model.meshes.len(),
            1,
            "fixture expects one object: {}",
            piece.resource
        );
        let mesh = &model.meshes[0];
        // Explicit fixture restriction: ASCII names and distinct raw map keys.
        // This does not implement arbitrary native material-name decoding.
        let mut metadata = NativeBodyFaceMetadata {
            smoothing: mesh.smoothing.clone().unwrap_or_default(),
            ..Default::default()
        };
        for material in &mesh.materials {
            assert!(material.name.is_ascii());
            let name = material.name.iter().map(|byte| u16::from(*byte)).collect();
            assert!(
                metadata
                    .materials
                    .insert(name, material.faces.clone())
                    .is_none()
            );
        }
        let mut source = mesh
            .body_merge_geometry(
                ModelVariant::N,
                &piece.geometry,
                piece.bones.map(|bone| i32::try_from(bone).unwrap()),
                metadata,
            )
            .unwrap();
        total_faces += source.faces.len();
        resources += 1;
        if let Some(destination) = destination.as_mut() {
            let report = destination
                .append_head_geometry_stages(
                    &mut source,
                    ModelVariant::N,
                    piece.merge_tolerance.expect("head merge tolerance"),
                )
                .unwrap();
            updated_corners += report.updated_corners;
            incomplete += usize::from(!report.fully_connected);
        } else {
            // Seed represented geometry only, not a fabricated CopyInstance.
            destination = Some(source);
        }
    }
    let destination = destination.unwrap();
    assert_eq!(resources, 5);
    assert_eq!(destination.faces.len(), total_faces);
    assert!(destination.object.field_369);
    assert!(
        destination
            .object
            .vertices
            .iter()
            .flat_map(|v| v.position)
            .all(f32::is_finite)
    );
    assert!(
        destination
            .faces
            .iter()
            .flat_map(|f| f.vertices)
            .all(|i| usize::from(i) < destination.object.vertices.len())
    );
    eprintln!(
        "Head geometry stages: {resources} mounted resources, {} vertices, {total_faces} faces, {updated_corners} corner updates, {incomplete} incomplete-connection reports; not full instance/cache/rendering parity",
        destination.object.vertices.len()
    );
}
