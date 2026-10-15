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
