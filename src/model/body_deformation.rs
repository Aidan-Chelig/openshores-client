//! Native anatomical ApplyScaleFactor branches, preserving connection correction.
use super::{
    Mesh, ModelVariant, NativeBodyObjectGroups, NativeBodyObjectScale, NativeBodyVertexSelection,
};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeBodyDeformation {
    pub base: NativeBodyObjectScale,
    pub field_178: f64,
    pub field_180: f64,
    pub field_188: f64,
}

impl NativeBodyDeformation {
    /// Tail/Head/Neck/Foot branches and the proven no-membership branch.
    /// Other branches are errors, NEVER successful uniform-scale substitutes.
    pub fn apply_vertex(
        &self,
        point: [f64; 3],
        selected: &NativeBodyVertexSelection,
    ) -> io::Result<[f64; 3]> {
        if !selected.any_match {
            return Ok(self.base.apply_unweighted_vertex(point));
        }
        // Native precedence is Tail, Head, Neck, Hand, Foot, Torso, then
        // top/bottom-only paths. Higher-priority unsupported Hand must not
        // accidentally fall through to a supported Foot group.
        let (pivot, uniform, vertical) = if let Some(pivot) = selected.anatomical[4] {
            (pivot, false, true)
        } else if let Some(pivot) = selected.anatomical[5] {
            (pivot, true, false)
        } else if let Some(pivot) = selected.anatomical[3] {
            (pivot, false, true)
        } else if selected.anatomical[1].is_some() {
            return Err(unsupported());
        } else if let Some(pivot) = selected.anatomical[2] {
            (pivot, true, true)
        } else {
            return Err(unsupported());
        };
        let scale = self.base.field_140;
        let (mut vertex, mut connection) = if scale == 0. {
            ([0.; 3], [0.; 3])
        } else {
            (
                point.map(|value| value * scale),
                self.base.fields_1f4.map(|value| f64::from(value) * scale),
            )
        };
        if self.field_180 != 1. {
            let scales = [
                self.field_180,
                if uniform { self.field_180 } else { 1. },
                self.field_180,
            ];
            around(&mut vertex, pivot, scales);
            around(&mut connection, pivot, scales);
        }
        if vertical && self.field_188 != 1. {
            around(&mut vertex, pivot, [1., self.field_188, 1.]);
            around(&mut connection, pivot, [1., self.field_188, 1.]);
        }
        // Shared581->5cd: anatomical membership reaches the additional
        // connection-only Y operation only when Top AND Wing matched.
        if self.field_178 != 1.
            && selected.wing
            && let Some(top) = selected.top
        {
            around(&mut connection, top, [1., self.field_178, 1.]);
        }
        let offset = self.base.fields_1c0.map(f64::from);
        Ok(std::array::from_fn(|axis| {
            (offset[axis] - connection[axis]) + vertex[axis]
        }))
    }
}

impl Mesh {
    /// Loaded coordinates, object metadata, per-vertex selection and native CPU
    /// deformation. Top/bottom inputs are initialized object values; serialized
    /// control-point writes override them. Unsupported vertices fail the whole
    /// result; no partially deformed mesh is published as success.
    pub fn body_deformed_positions(
        &self,
        variant: ModelVariant,
        loader_scale: f64,
        definition: &NativeBodyDeformation,
        top_scale: [f32; 3],
        bottom_scale: [f32; 3],
    ) -> io::Result<Vec<[f64; 3]>> {
        let groups = NativeBodyObjectGroups::from_mesh(self)?;
        let mut definition = *definition;
        if let Some(connection) = groups.control_point(0x1cc) {
            definition.base.fields_1f4 = connection;
        }
        let top = groups.control_point(0x268).unwrap_or(top_scale);
        let bottom = groups.control_point(0x2d0).unwrap_or(bottom_scale);
        self.loaded_positions(variant, loader_scale)?
            .into_iter()
            .map(|point| definition.apply_vertex(point, &groups.select_vertex(point, top, bottom)?))
            .collect()
    }
}

fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "native body deformation branch is not implemented",
    )
}

fn around(point: &mut [f64; 3], pivot: [f64; 3], scale: [f64; 3]) {
    for axis in 0..3 {
        // Keep native subtract/add even for unchanged axes within a taken
        // branch. Do not simplify cancellation or replace with an FMA.
        let delta = point[axis] - pivot[axis];
        point[axis] = if scale[axis] == 1. {
            delta + pivot[axis]
        } else {
            delta * scale[axis] + pivot[axis]
        };
    }
}

#[cfg(test)]
mod tests;
