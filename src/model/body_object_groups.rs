//! LoadMeshChunk's ordered control-point writes and retained group list.
//! Empty state means no observed writes, NOT reconstructed object defaults.
use super::{Mesh, NativeBodyVertexGroup};
use std::{collections::BTreeMap, io};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeBodyObjectGroups {
    point_writes: BTreeMap<u16, [f32; 3]>,
    retained: Vec<NativeBodyVertexGroup>,
    sets_3a9: bool,
}

impl NativeBodyObjectGroups {
    pub fn from_mesh(mesh: &Mesh) -> io::Result<Self> {
        let mut result = Self::default();
        result.apply_mesh_groups(mesh)?;
        Ok(result)
    }

    /// Apply groups in file order. Earlier writes survive a later Rust decode
    /// error; raw input stays unchanged. This does not clear existing state.
    pub fn apply_mesh_groups(&mut self, mesh: &Mesh) -> io::Result<()> {
        for raw in &mesh.vertex_groups {
            self.apply_group(NativeBodyVertexGroup::from_group(raw)?);
        }
        Ok(())
    }

    pub fn apply_group(&mut self, group: NativeBodyVertexGroup) {
        let destination = match group.kind_34 {
            3 => Some(0x1cc),
            4 => Some(0x200),
            10 => Some(0x234),
            11 => Some(0x304),
            12 => Some(0x268),
            13 => Some(0x29c),
            14 => Some(0x2d0),
            19 => Some(0x3ac),
            20 => Some(0x3e0),
            _ => None,
        };
        if let Some(destination) = destination {
            // Native copies an Au3dsGLSLVertex whose xyz fields begin at+28.
            // Represent the xyz write, not invented normal/UV/bone semantics.
            self.point_writes.insert(destination, group.point_28);
            if group.kind_34 != 4 {
                return;
            }
        }
        if group.kind_34 == 1 {
            self.sets_3a9 = true;
        }
        self.retained.push(group);
    }

    /// Au3dsGLSLVertex base offset, e.g.1cc gives xyz at object1f4/1f8/1fc.
    /// None means this stream did not write the field, not a zero coordinate.
    pub fn control_point(&self, vertex_offset: u16) -> Option<[f32; 3]> {
        self.point_writes.get(&vertex_offset).copied()
    }

    pub fn retained_groups(&self) -> &[NativeBodyVertexGroup] {
        &self.retained
    }
    pub fn sets_field_3a9(&self) -> bool {
        self.sets_3a9
    }
}

#[cfg(test)]
mod tests;
