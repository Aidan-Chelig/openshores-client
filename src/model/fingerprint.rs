use super::*;

impl Model {
    /// Independent parity fingerprint for raw custom vertex-group records.
    /// Existing geometry/material fingerprints intentionally stay unchanged.
    pub fn vertex_group_digest(&self) -> String {
        use sha1::{Digest, Sha1};
        let mut hash = Sha1::new();
        hash.update((self.meshes.len() as u32).to_le_bytes());
        for mesh in &self.meshes {
            hash.update((mesh.vertex_groups.len() as u32).to_le_bytes());
            for group in &mesh.vertex_groups {
                hash.update([u8::from(group.name.is_some())]);
                if let Some(name) = &group.name {
                    hash.update((name.len() as u32).to_le_bytes());
                    hash.update(name);
                }
                hash.update([u8::from(group.members.is_some())]);
                if let Some(members) = &group.members {
                    hash.update((members.len() as u32).to_le_bytes());
                    for member in members {
                        for value in member.position.into_iter().chain([member.weight]) {
                            hash.update(value.to_le_bytes());
                        }
                    }
                }
            }
        }
        format!("{:x}", hash.finalize())
    }

    /// Canonical raw-material parity fingerprint; no shading/gamma conversion.
    pub fn material_digest(&self) -> String {
        use sha1::{Digest, Sha1};
        let mut hash = Sha1::new();
        fn length(hash: &mut Sha1, count: usize) {
            hash.update((count as u32).to_le_bytes());
        }
        fn name(hash: &mut Sha1, value: &[u8]) {
            length(hash, value.len());
            hash.update(value);
        }
        length(&mut hash, self.materials.len());
        for material in &self.materials {
            hash.update([u8::from(material.name.is_some())]);
            if let Some(value) = &material.name {
                name(&mut hash, value);
            }
            length(&mut hash, material.textures.len());
            for texture in &material.textures {
                name(&mut hash, texture);
            }
            length(&mut hash, material.colors.len());
            for color in &material.colors {
                hash.update(color.property.to_le_bytes());
                match color.value {
                    ColorValue::RgbFloat(values) => {
                        hash.update(0x0010u16.to_le_bytes());
                        for value in values {
                            hash.update(value.to_le_bytes());
                        }
                    }
                    ColorValue::Rgb8(values) => {
                        hash.update(0x0011u16.to_le_bytes());
                        hash.update(values);
                    }
                    ColorValue::LinearRgb8(values) => {
                        hash.update(0x0012u16.to_le_bytes());
                        hash.update(values);
                    }
                }
            }
            length(&mut hash, material.amounts.len());
            for amount in &material.amounts {
                hash.update(amount.property.to_le_bytes());
                match amount.value {
                    AmountValue::Word(value) => {
                        hash.update([2]);
                        hash.update(value.to_le_bytes());
                    }
                    AmountValue::Float(value) => {
                        hash.update([4]);
                        hash.update(value.to_le_bytes());
                    }
                }
            }
            hash.update([
                u8::from(material.two_sided),
                u8::from(material.shading.is_some()),
            ]);
            if let Some(value) = material.shading {
                hash.update(value.to_le_bytes());
            }
        }
        format!("{:x}", hash.finalize())
    }

    /// Stable SHA-1 for parity tests, not an authenticity/security guarantee.
    /// Includes raw-coordinate geometry in received order, not rendered geometry.
    pub fn geometry_digest(&self) -> String {
        use sha1::{Digest, Sha1};
        let mut hash = Sha1::new();
        fn length(hash: &mut Sha1, count: usize) {
            hash.update((count as u32).to_le_bytes());
        }
        fn floats<const N: usize>(hash: &mut Sha1, values: &[f32; N]) {
            for value in values {
                hash.update(value.to_le_bytes());
            }
        }
        length(&mut hash, self.meshes.len());
        for mesh in &self.meshes {
            length(&mut hash, mesh.name.len());
            hash.update(&mesh.name);
            length(&mut hash, mesh.vertices.len());
            for vertex in &mesh.vertices {
                floats(&mut hash, vertex);
            }
            length(&mut hash, mesh.faces.len());
            for face in &mesh.faces {
                for vertex in face.vertices {
                    hash.update(vertex.to_le_bytes());
                }
                hash.update(face.flags.to_le_bytes());
                hash.update([u8::from(face.extension.is_some())]);
                if let Some(values) = face.extension {
                    floats(&mut hash, &values);
                }
                // Extend the parity serialization only for NT faces; retain
                // every established Standard/N/X fingerprint unchanged.
                if let Some(values) = face.nt_extension {
                    hash.update(b"NT");
                    floats(&mut hash, &values);
                }
            }
            hash.update([u8::from(mesh.uvs.is_some())]);
            if let Some(uvs) = &mesh.uvs {
                length(&mut hash, uvs.len());
                for uv in uvs {
                    floats(&mut hash, uv);
                }
            }
            hash.update([u8::from(mesh.matrix.is_some())]);
            if let Some(matrix) = mesh.matrix {
                floats(&mut hash, &matrix);
            }
            hash.update([u8::from(mesh.smoothing.is_some())]);
            if let Some(smoothing) = &mesh.smoothing {
                length(&mut hash, smoothing.len());
                for value in smoothing {
                    hash.update(value.to_le_bytes());
                }
            }
            length(&mut hash, mesh.materials.len());
            for material in &mesh.materials {
                length(&mut hash, material.name.len());
                hash.update(&material.name);
                length(&mut hash, material.faces.len());
                for face in &material.faces {
                    hash.update(face.to_le_bytes());
                }
            }
        }
        format!("{:x}", hash.finalize())
    }
}
