//! Bounded, raw-coordinate 3DS-family geometry. No renderer conversion is implied.
use std::{collections::BTreeMap, io, ops::Range};

const MAX_INPUT: usize = 64 * 1024 * 1024;
const MAX_ELEMENTS: usize = 1_000_000;
const MAX_CHUNKS: usize = 100_000;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

mod body_scale;
mod coordinates;
pub use body_scale::NativeBodyObjectScale;
mod body_bone_names;
mod body_merge;
mod body_merge_bake;
pub use body_merge_bake::NativeBodyMergeBake;
mod body_mesh;
pub use body_merge::{NativeBodyStoredVertex, native_body_add_if_unique};
mod body_weights;
pub use body_mesh::NativeBodyMeshGeometry;
pub use body_weights::NativeBodyVertexWeights;
mod body_groups;
pub use body_bone_names::{native_body_is_bone, native_body_neck_bone, native_body_tail_bone};
pub use body_groups::NativeBodyVertexGroup;
mod body_object_groups;
pub use body_object_groups::NativeBodyObjectGroups;
mod body_selection;
pub use body_selection::NativeBodyVertexSelection;
mod body_deformation;
pub use body_deformation::NativeBodyDeformation;
mod body_defaults;
mod body_definition;
pub use body_definition::NativeBodyModelGeometry;
mod decode;
mod fingerprint;
mod roles;
pub use roles::MeshRole;
#[cfg(test)]
mod groups_tests;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelVariant {
    Standard,
    N,
    NT,
    X,
}

impl ModelVariant {
    pub fn from_name(name: &str) -> io::Result<Self> {
        match name.rsplit('.').next() {
            Some("3ds") => Ok(Self::Standard),
            Some("3dsN") => Ok(Self::N),
            Some("3dsNT") => Ok(Self::NT),
            Some("3dsX") => Ok(Self::X),
            _ => Err(invalid("unsupported model variant")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub vertices: [u16; 3],
    pub flags: u16,
    /// Nine serialized extension floats for N/NT; semantic interpretation pending.
    pub extension: Option<[f32; 9]>,
    /// Ten additional serialized floats for NT, in source order.
    pub nt_extension: Option<[f32; 10]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaceMaterial {
    pub name: Vec<u8>,
    pub faces: Vec<u16>,
}

#[derive(Default, Debug)]
pub struct Mesh {
    pub name: Vec<u8>,
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<Face>,
    pub uvs: Option<Vec<[f32; 2]>>,
    pub matrix: Option<[f32; 12]>,
    pub smoothing: Option<Vec<u32>>,
    pub materials: Vec<FaceMaterial>,
    /// Source order affects native loading, particularly X matrices.
    pub geometry_order: Vec<GeometryChunk>,
    /// Custom 0x4112 groups: raw names and weighted positions, not animation.
    pub vertex_groups: Vec<VertexGroup>,
}

#[derive(Default, Debug)]
pub struct VertexGroup {
    pub name: Option<Vec<u8>>,
    pub members: Option<Vec<GroupVertex>>,
    /// Native metadata assignment depends on whether names precede members.
    pub field_order: Vec<VertexGroupChunk>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexGroupChunk {
    Name,
    Members,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupVertex {
    pub position: [f32; 3],
    pub weight: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryChunk {
    Vertices,
    Matrix,
}

#[derive(Default, Debug)]
pub struct Material {
    pub name: Option<Vec<u8>>,
    pub textures: Vec<Vec<u8>>,
    /// Ordered encodings, not a selected/gamma-converted renderer color.
    pub colors: Vec<ColorRecord>,
    /// Parent chunk IDs retain the original property identity.
    pub amounts: Vec<AmountRecord>,
    pub two_sided: bool,
    pub shading: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColorValue {
    RgbFloat([f32; 3]),
    Rgb8([u8; 3]),
    LinearRgb8([u8; 3]),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorRecord {
    /// 0xa010 ambient, 0xa020 diffuse, 0xa030 specular.
    pub property: u16,
    pub value: ColorValue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AmountValue {
    Word(u16),
    Float(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AmountRecord {
    pub property: u16,
    pub value: AmountValue,
}

#[derive(Debug)]
pub struct OpaqueChunk {
    pub id: u16,
    pub range: Range<usize>,
}

#[derive(Default, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub version: Option<u32>,
    pub master_scale: Option<f32>,
    pub chunks: BTreeMap<u16, usize>,
    pub opaque: Vec<OpaqueChunk>,
    /// Retained bytes make opaque chunk ranges lossless, including keyframes.
    pub source: Vec<u8>,
}
