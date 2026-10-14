//! Mounted item mesh coordinates. Entity orientation/material fidelity pending.
use super::*;
use bevy::ecs::system::SystemParam;
use std::collections::{BTreeMap, VecDeque};

#[derive(Component)]
pub(super) struct MountedItemModel(pub String);

#[derive(Resource, Default)]
pub(super) struct Cache {
    stamp: (u64, u64),
    models: BTreeMap<String, Option<Handle<Mesh>>>,
    recent: VecDeque<String>,
}
impl Cache {
    fn reset(&mut self, stamp: (u64, u64)) {
        if self.stamp != stamp {
            self.models.clear();
            self.recent.clear();
            self.stamp = stamp;
        }
    }
    fn get(&mut self, resource: &str) -> Option<Option<Handle<Mesh>>> {
        let result = self.models.get(resource)?.clone();
        self.recent.retain(|key| key != resource);
        self.recent.push_back(resource.to_owned());
        Some(result)
    }
    fn insert(&mut self, resource: String, model: Option<Handle<Mesh>>) {
        self.recent.retain(|key| key != &resource);
        self.recent.push_back(resource.clone());
        self.models.insert(resource, model);
        while self.models.len() > 64 {
            if let Some(key) = self.recent.pop_front() {
                self.models.remove(&key);
            }
        }
    }
}
#[derive(SystemParam)]
pub(super) struct Rendering<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub setup: Res<'w, assets::Setup>,
    pub cache: ResMut<'w, Cache>,
}

fn geometry(
    model: &crate::model::Model,
    variant: crate::model::ModelVariant,
    scale: f64,
) -> io::Result<Mesh> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(io::Error::other("invalid model display scale"));
    }
    crate::model_preview::bounds(model)?;
    let mut positions = Vec::new();
    let mut triangles = Vec::new();
    for (part, role) in model.meshes.iter().zip(model.mesh_roles(variant)) {
        if role != crate::model::MeshRole::Geometry || part.faces.is_empty() {
            continue;
        }
        let offset = positions.len() as u32;
        // The model loader's caller scale remains explicitly 1.0 here. Planet
        // display scaling happens AFTER native float-storage/local-frame stages.
        for point in part.loaded_positions(variant, 1.0)? {
            let point = point.map(|value| (value * scale) as f32);
            if point.iter().any(|value| !value.is_finite()) {
                return Err(io::Error::other("non-finite rendered model"));
            }
            positions.push(point);
        }
        triangles.extend(
            part.faces
                .iter()
                .map(|face| face.vertices.map(|index| offset + u32::from(index))),
        );
    }
    // Successfully decoded metadata-only models intentionally render nothing;
    // they must not be misreported as unavailable and turned into spheres.
    Ok(crate::viewer::smooth_mesh(&positions, &triangles))
}

#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;

impl Rendering<'_> {
    pub(super) fn model(&mut self, resource: &str, scale: f64) -> Option<Handle<Mesh>> {
        let stamp = (self.setup.revision, scale.to_bits());
        self.cache.reset(stamp);
        if let Some(model) = self.cache.get(resource) {
            return model;
        }
        let prepared = self.setup.prepared.as_ref()?;
        let result = (|| {
            let bytes = prepared.mount.read_resource(resource)?;
            let variant = crate::model::ModelVariant::from_name(resource)?;
            let model = crate::model::Model::decode(&bytes, variant)?;
            geometry(&model, variant, scale)
        })();
        let handle = match result {
            Ok(mesh) => {
                println!(
                    "Item model loaded from user installation: {resource} (mesh coordinate stages)"
                );
                Some(self.meshes.add(mesh))
            }
            Err(error) => {
                eprintln!("Item model unavailable: {resource}: {error}");
                None
            }
        };
        self.cache.insert(resource.into(), handle.clone());
        handle
    }
}
