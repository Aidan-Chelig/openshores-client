//! Native HandLength metadata and arithmetic, not skinned/rendered geometry.
use super::{NativeDhDna, native_dp_size_factor};
use crate::model::{MeshRole, Model, ModelVariant, VertexGroup, VertexGroupChunk};
use std::io;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeDpHandGeometry {
    minimum: [f64; 3],
    maximum: [f64; 3],
    connect: [f64; 3],
    pivot: [f64; 3],
}

fn contains(name: &[u8], word: &[u8]) -> bool {
    name.windows(word.len()).any(|part| part == word)
}

// Relevant prefix of VGroup::LoadChunk's ordered classification. ASCII tokens
// in Qt's raw byte-to-QChar names, not case-folded Unicode names.
fn point_kind(name: &[u8]) -> u8 {
    if contains(name, b"ScalePt") {
        return 0;
    }
    if contains(name, b"ConnectPt") {
        return 1;
    }
    if contains(name, b"MidPt")
        || matches!(name, b"Top" | b"Bottom")
        || contains(name, b"BottomSkin")
        || matches!(
            name,
            b"TopConnect" | b"BottomConnect" | b"HandWingPt" | b"EyePosition"
        )
        || contains(name, b"Torso")
    {
        return 0;
    }
    if contains(name, b"Hand") { 2 } else { 0 }
}

fn point(group: &VertexGroup) -> io::Result<(u8, [f64; 3])> {
    if group
        .field_order
        .iter()
        .filter(|field| **field == VertexGroupChunk::Name)
        .count()
        != usize::from(group.name.is_some())
        || group
            .field_order
            .iter()
            .filter(|field| **field == VertexGroupChunk::Members)
            .count()
            != usize::from(group.members.is_some())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing native vertex-group field order",
        ));
    }
    let mut kind = 0;
    let mut position = [0.0; 3];
    for field in &group.field_order {
        match field {
            VertexGroupChunk::Name => {
                kind = point_kind(
                    group
                        .name
                        .as_deref()
                        .ok_or_else(|| io::Error::other("missing group name"))?,
                )
            }
            VertexGroupChunk::Members => {
                let members = group
                    .members
                    .as_ref()
                    .ok_or_else(|| io::Error::other("missing group members"))?;
                // Single-record fast path assigns ConnectPt but not Hand.
                // Multi-record paths use the final serialized point, not a
                // weighted centroid or the sorted map's final key.
                if (kind == 1 || (kind == 2 && members.len() > 1))
                    && let Some(last) = members.last()
                {
                    position = last.position.map(f64::from);
                }
            }
        }
    }
    Ok((kind, position))
}

impl NativeDpHandGeometry {
    /// Native N-model extent/first-retained-object metadata at loader scale1.
    /// Missing/unsupported metadata stays an error, not guessed primitive data.
    pub fn from_model(model: &Model) -> io::Result<Self> {
        let roles = model.mesh_roles(ModelVariant::N);
        let mut bounds: Option<([f64; 3], [f64; 3])> = None;
        let mut first = None;
        for (mesh, role) in model.meshes.iter().zip(roles) {
            if role == MeshRole::Rejected {
                continue;
            }
            if !mesh.name.starts_with(b"-")
                && !matches!(role, MeshRole::Station { .. } | MeshRole::Weapon { .. })
            {
                for p in mesh.loaded_positions(ModelVariant::N, 1.0)? {
                    if let Some((min, max)) = &mut bounds {
                        for i in 0..3 {
                            if min[i] > p[i] {
                                min[i] = p[i];
                            }
                            if p[i] > max[i] {
                                max[i] = p[i];
                            }
                        }
                    } else {
                        bounds = Some((p, p));
                    }
                }
            }
            if first.is_none()
                && !matches!(
                    role,
                    MeshRole::Station { .. } | MeshRole::Weapon { .. } | MeshRole::UpVector
                )
            {
                if role == MeshRole::Screens {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "screen-bearing native hand model",
                    ));
                }
                let mut connect = [0.0; 3];
                let mut pivot = None;
                for group in &mesh.vertex_groups {
                    let (kind, p) = point(group)?;
                    if kind == 1 {
                        connect = p;
                    }
                    if kind == 2 && pivot.is_none() {
                        pivot = Some(p);
                    }
                }
                first = Some((connect, pivot.unwrap_or([0.0; 3])));
            }
        }
        let (minimum, maximum) = bounds.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "native hand model has no extent-bearing object",
            )
        })?;
        let (connect, pivot) = first.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "native hand model has no retained object",
            )
        })?;
        Ok(Self {
            minimum,
            maximum,
            connect,
            pivot,
        })
    }

    /// AuGlobal's cached hand offset is [0, hand length, z offset]. Native
    /// HandLength returns its Y component narrowed to f32, not a void return.
    pub fn offset_for_dna(&self, dna: &NativeDhDna) -> [f64; 3] {
        let scale = f64::from(native_dp_size_factor(dna) / f32::from_bits(0x3f57_0a3d));
        let mut minimum = self.minimum.map(|v| v * scale);
        let mut maximum = self.maximum.map(|v| v * scale);
        let mut connect = self.connect.map(|v| v * scale);
        let a = (f32::from(dna.0[8] >> 3) * 0.0625 + 1.0) * 0.25;
        let b = f32::from(dna.0[6] >> 3) * 0.0625 + 0.5;
        let broad = ((f64::from(a) + f64::from(a)) * (f64::from(b) / 1.5)) as f32;
        let width = (f32::from(dna.0[11] & 31) * 0.0625 + 0.5) * 0.25;
        let width = width + width;
        // Native pivot is NOT multiplied by the initial size scale.
        for p in [&mut minimum, &mut maximum, &mut connect] {
            if broad != 1.0 {
                for (v, pivot) in p.iter_mut().zip(self.pivot) {
                    *v = (*v - pivot) * f64::from(broad) + pivot;
                }
            }
            if width != 1.0 {
                for (i, v) in p.iter_mut().enumerate() {
                    let delta = *v - self.pivot[i];
                    *v = (if i == 1 {
                        delta * f64::from(width)
                    } else {
                        delta
                    }) + self.pivot[i];
                }
            }
        }
        [0.0, maximum[1] - connect[1], connect[2] - minimum[2]]
    }
}

#[cfg(test)]
mod tests;
