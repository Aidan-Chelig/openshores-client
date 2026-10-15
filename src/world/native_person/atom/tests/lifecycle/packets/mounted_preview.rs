//! Real mounted meshes through a dispatched Person's explicit preview boundary.
//! Fixture DNA is not claimed to have been received or natively merged.
use super::*;
use crate::{
    asset_mount::AssetMount,
    model::{Model, ModelVariant},
    world::{
        native_body::{NativeDpHandCache, NativeDpInitialization, NativeDpSkinPlan},
        native_dna::NativeDhDna,
    },
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; offline, no GPU window"]
fn mounted_body_parts_reach_dispatched_person_preview_meshes() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let profile = FloatEncoding::TypeWidths;
    let mut harness = Harness::new(profile);
    let outcome = harness.receive(&complete(profile)).unwrap().unwrap();
    let avatar =
        crate::client_model::NativeAvatarPresentation::from_dispatched(&harness.people, outcome)
            .unwrap();
    assert!(avatar.model_space_geometry().is_none());
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[12] = 4;
    dna.0[14] = 0x51;
    dna.0[20] = 0x40;
    let mut body = NativeDpInitialization::begin(&dna);
    body.finish_part_calculations(0, &mut NativeDpHandCache::new(&mount))
        .unwrap();
    let mut geometry = Vec::new();
    let mut resources = 0;
    for planned in body.skin_geometry_plans(0, 0, [1.; 4]).unwrap() {
        // Explicitly test individual fallback PARTS, not successful native merges.
        let requests: Vec<_> = match planned.plan {
            NativeDpSkinPlan::Arm(parts) => parts
                .into_iter()
                .map(|p| (p.resource, p.geometry, p.bones))
                .collect(),
            NativeDpSkinPlan::Leg(parts) => parts
                .into_iter()
                .map(|p| (p.resource, p.geometry, p.bones))
                .collect(),
            NativeDpSkinPlan::Tail(part) => part
                .into_iter()
                .map(|p| (p.resource, p.geometry, p.bones))
                .collect(),
            NativeDpSkinPlan::Torso(plan) => plan
                .into_iter()
                .flat_map(|p| p.fallback)
                .map(|p| (p.resource, p.geometry, p.bones))
                .collect(),
            NativeDpSkinPlan::Head(plan) => plan
                .fallback
                .into_iter()
                .chain(plan.neck)
                .map(|p| (p.resource, p.geometry, p.bones))
                .collect(),
        };
        for (resource, definition, bones) in requests {
            let model =
                Model::decode(&mount.read_resource(&resource).unwrap(), ModelVariant::N).unwrap();
            resources += 1;
            for mesh in model.meshes {
                geometry.push(
                    mesh.body_defined_geometry(
                        ModelVariant::N,
                        1.,
                        &definition,
                        bones.map(|b| i32::try_from(b).unwrap()),
                    )
                    .unwrap(),
                );
            }
        }
    }
    assert_eq!(resources, 17);
    assert_eq!(
        geometry.iter().map(|p| p.positions.len()).sum::<usize>(),
        3410
    );
    let avatar = avatar.with_model_space_geometry(geometry).unwrap();
    crate::client_ui::assert_native_geometry_preview(avatar);
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; offline merged-geometry preview"]
fn mounted_merged_head_and_torso_reach_dispatched_person_preview_meshes() {
    use crate::model::{NativeBodyFaceMetadata, NativeBodyMergeGeometry};
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let profile = FloatEncoding::TypeWidths;
    let mut harness = Harness::new(profile);
    let outcome = harness.receive(&complete(profile)).unwrap().unwrap();
    let avatar =
        crate::client_model::NativeAvatarPresentation::from_dispatched(&harness.people, outcome)
            .unwrap();
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[12] = 4;
    dna.0[14] = 0x51;
    dna.0[20] = 0x60;
    let mut body = NativeDpInitialization::begin(&dna);
    body.finish_part_calculations(0, &mut NativeDpHandCache::new(&mount))
        .unwrap();
    let mut previews = Vec::new();
    for planned in body.skin_geometry_plans(0, 0, [1.; 4]).unwrap() {
        let (pieces, head): (Vec<_>, bool) = match planned.plan {
            NativeDpSkinPlan::Head(plan) => (
                plan.fallback
                    .into_iter()
                    .map(|piece| {
                        (
                            piece.resource,
                            piece.geometry,
                            piece.bones,
                            piece.merge_tolerance.unwrap(),
                        )
                    })
                    .collect(),
                true,
            ),
            NativeDpSkinPlan::Torso(Some(plan)) => (
                plan.fallback
                    .into_iter()
                    .map(|piece| {
                        assert!(!piece.crest);
                        (
                            piece.resource,
                            piece.geometry,
                            piece.bones,
                            plan.merge_tolerance,
                        )
                    })
                    .collect(),
                false,
            ),
            _ => continue,
        };
        let mut destination: Option<NativeBodyMergeGeometry> = None;
        let mut projection_source = None;
        for (resource, definition, bones, tolerance) in pieces {
            let model =
                Model::decode(&mount.read_resource(&resource).unwrap(), ModelVariant::N).unwrap();
            assert_eq!(model.meshes.len(), 1);
            let mesh = &model.meshes[0];
            let mut metadata = NativeBodyFaceMetadata {
                smoothing: mesh.smoothing.clone().unwrap_or_default(),
                ..Default::default()
            };
            for material in &mesh.materials {
                assert!(material.name.is_ascii());
                assert!(
                    metadata
                        .materials
                        .insert(
                            material.name.iter().map(|byte| u16::from(*byte)).collect(),
                            material.faces.clone()
                        )
                        .is_none()
                );
            }
            let mut source = mesh
                .body_merge_geometry(
                    ModelVariant::N,
                    &definition,
                    bones.map(|bone| i32::try_from(bone).unwrap()),
                    metadata,
                )
                .unwrap();
            if let Some(destination) = destination.as_mut() {
                if head {
                    destination
                        .append_head_geometry_stages(&mut source, ModelVariant::N, tolerance)
                        .unwrap();
                } else {
                    destination
                        .append_torso_geometry_stages(&mut source, ModelVariant::N, tolerance)
                        .unwrap();
                }
            } else {
                destination = Some(source);
                projection_source = model.meshes.into_iter().next();
            }
        }
        previews.push(
            destination
                .unwrap()
                .untextured_baked_preview_geometry(&projection_source.unwrap(), ModelVariant::N)
                .unwrap(),
        );
    }
    assert_eq!(previews.len(), 2);
    assert_eq!(
        previews.iter().map(|mesh| mesh.faces.len()).sum::<usize>(),
        2224
    );
    crate::client_ui::assert_native_geometry_preview(
        avatar.with_model_space_geometry(previews).unwrap(),
    );
}
