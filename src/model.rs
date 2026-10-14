//! Bounded, raw-coordinate 3DS-family geometry. No renderer conversion is implied.
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    ops::Range,
};

const MAX_INPUT: usize = 64 * 1024 * 1024;
const MAX_ELEMENTS: usize = 1_000_000;
const MAX_CHUNKS: usize = 100_000;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn chunk(id: u16, body: &[u8]) -> Vec<u8> {
        let mut bytes = id.to_le_bytes().to_vec();
        bytes.extend(((body.len() + 6) as u32).to_le_bytes());
        bytes.extend(body);
        bytes
    }
    fn fixture(variant: ModelVariant, bad_vertex: bool, bad_material: bool) -> Vec<u8> {
        let mut vertices = 3u16.to_le_bytes().to_vec();
        for value in [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
            vertices.extend(value.to_le_bytes());
        }
        let mut faces = 1u16.to_le_bytes().to_vec();
        for value in [0u16, 1, if bad_vertex { 3 } else { 2 }, 7] {
            faces.extend(value.to_le_bytes());
        }
        if variant == ModelVariant::N {
            for value in 0..9 {
                faces.extend((value as f32).to_le_bytes());
            }
        }
        faces.extend(chunk(0x4150, &42u32.to_le_bytes()));
        let mut assignment = b"mat\0".to_vec();
        assignment.extend(1u16.to_le_bytes());
        assignment.extend(u16::from(bad_material).to_le_bytes());
        faces.extend(chunk(0x4130, &assignment));
        let mut mesh = chunk(0x4110, &vertices);
        mesh.extend(chunk(0x4120, &faces));
        // Deliberately unequal UV/vertex counts are preserved, as in reference assets.
        let mut uv = 1u16.to_le_bytes().to_vec();
        uv.extend(0.25f32.to_le_bytes());
        uv.extend(0.75f32.to_le_bytes());
        mesh.extend(chunk(0x4140, &uv));
        let mut object = b"triangle\0".to_vec();
        object.extend(chunk(0x4100, &mesh));
        let mut editor = chunk(0x4000, &object);
        let mut material = chunk(0xa000, b"mat\0");
        material.extend(chunk(0xa200, &chunk(0xa300, b"texture.png\0")));
        editor.extend(chunk(0xafff, &material));
        let mut main = chunk(0x0002, &3u32.to_le_bytes());
        main.extend(chunk(0x3d3d, &editor));
        main.extend(chunk(0xbeef, b"opaque"));
        chunk(0x4d4d, &main)
    }

    #[test]
    fn three_variants_preserve_raw_geometry_and_opaque_bytes() {
        for variant in [ModelVariant::Standard, ModelVariant::N, ModelVariant::X] {
            let bytes = fixture(variant, false, false);
            let model = Model::decode(&bytes, variant).unwrap();
            assert_eq!(model.meshes[0].vertices[1], [1.0, 0.0, 0.0]);
            assert_eq!(model.meshes[0].faces[0].vertices, [0, 1, 2]);
            assert_eq!(
                model.meshes[0].faces[0].extension.is_some(),
                variant == ModelVariant::N
            );
            if let Some(values) = model.meshes[0].faces[0].extension {
                assert_eq!(values[8], 8.0);
            }
            assert_eq!(model.meshes[0].smoothing, Some(vec![42]));
            assert_eq!(model.meshes[0].uvs, Some(vec![[0.25, 0.75]]));
            assert_eq!(model.materials[0].textures, vec![b"texture.png".to_vec()]);
            let opaque = model
                .opaque
                .iter()
                .find(|chunk| chunk.id == 0xbeef)
                .unwrap();
            assert_eq!(
                &model.source[opaque.range.start + 6..opaque.range.end],
                b"opaque"
            );
            for end in 0..bytes.len() {
                assert!(Model::decode(&bytes[..end], variant).is_err());
            }
            let mut trailing = bytes.clone();
            trailing.push(0);
            assert!(Model::decode(&trailing, variant).is_err());
        }
    }

    #[test]
    fn indices_variants_and_nonfinite_values_are_rejected() {
        assert!(
            Model::decode(
                &fixture(ModelVariant::Standard, true, false),
                ModelVariant::Standard
            )
            .is_err()
        );
        assert!(
            Model::decode(
                &fixture(ModelVariant::Standard, false, true),
                ModelVariant::Standard
            )
            .is_err()
        );
        assert!(
            Model::decode(
                &fixture(ModelVariant::Standard, false, false),
                ModelVariant::N
            )
            .is_err()
        );
        assert!(
            Model::decode(
                &fixture(ModelVariant::N, false, false),
                ModelVariant::Standard
            )
            .is_err()
        );
        let bytes = chunk(
            0x4d4d,
            &chunk(0x3d3d, &chunk(0x0100, &f32::NAN.to_le_bytes())),
        );
        assert!(Model::decode(&bytes, ModelVariant::Standard).is_err());
        assert!(ModelVariant::from_name("module.SoH").is_err());
    }

    #[test]
    fn malformed_chunks_names_and_duplicate_empty_lists() {
        for body in [&[1u8][..], &[1, 0, 5, 0, 0, 0], &[1, 0, 255, 255, 255, 255]] {
            assert!(Model::decode(&chunk(0x4d4d, body), ModelVariant::Standard).is_err());
        }
        let unnamed = chunk(0x4d4d, &chunk(0x3d3d, &chunk(0x4000, b"no terminator")));
        assert!(Model::decode(&unnamed, ModelVariant::Standard).is_err());
        let mut mesh = chunk(0x4110, &[0, 0]);
        mesh.extend(chunk(0x4110, &[0, 0]));
        let mut object = b"empty\0".to_vec();
        object.extend(chunk(0x4100, &mesh));
        let duplicate = chunk(0x4d4d, &chunk(0x3d3d, &chunk(0x4000, &object)));
        assert!(Model::decode(&duplicate, ModelVariant::Standard).is_err());
        let too_many = chunk(0x4d4d, &chunk(0xbeef, &[]).repeat(MAX_CHUNKS));
        assert!(Model::decode(&too_many, ModelVariant::Standard).is_err());
    }

    fn material_fixture(body: &[u8]) -> Vec<u8> {
        chunk(0x4d4d, &chunk(0x3d3d, &chunk(0xafff, body)))
    }

    #[test]
    fn material_encodings_and_source_order_are_preserved() {
        let mut rgb = chunk(0x11, &[10, 20, 30]);
        rgb.extend(chunk(0x12, &[40, 50, 60]));
        // Unsupported linear-float colors stay opaque, matching the search boundary.
        rgb.extend(chunk(0x13, &[0; 12]));
        let mut material = chunk(0xa020, &rgb);
        let floats: Vec<_> = [0.25f32, 0.5, 0.75]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect();
        material.extend(chunk(0xa030, &chunk(0x10, &floats)));
        material.extend(chunk(0xa040, &chunk(0x30, &75u16.to_le_bytes())));
        material.extend(chunk(0xa050, &chunk(0x30, &0.375f32.to_le_bytes())));
        material.extend(chunk(0xa081, &[]));
        material.extend(chunk(0xa100, &3u16.to_le_bytes()));
        let model = Model::decode(&material_fixture(&material), ModelVariant::N).unwrap();
        let decoded = &model.materials[0];
        assert_eq!(
            decoded.colors,
            vec![
                ColorRecord {
                    property: 0xa020,
                    value: ColorValue::Rgb8([10, 20, 30])
                },
                ColorRecord {
                    property: 0xa020,
                    value: ColorValue::LinearRgb8([40, 50, 60])
                },
                ColorRecord {
                    property: 0xa030,
                    value: ColorValue::RgbFloat([0.25, 0.5, 0.75])
                },
            ]
        );
        assert_eq!(
            decoded.amounts,
            vec![
                AmountRecord {
                    property: 0xa040,
                    value: AmountValue::Word(75)
                },
                AmountRecord {
                    property: 0xa050,
                    value: AmountValue::Float(0.375)
                },
            ]
        );
        assert!(decoded.two_sided);
        assert_eq!(decoded.shading, Some(3));
        assert!(model.opaque.iter().any(|chunk| chunk.id == 0x13));
        assert!(Model::decode(&material_fixture(&material), ModelVariant::X).is_err());
        assert_ne!(model.geometry_digest(), model.material_digest());
    }

    #[test]
    fn malformed_material_values_are_rejected() {
        for material in [
            chunk(0xa020, &chunk(0x11, &[1, 2])),
            chunk(0xa020, &chunk(0x12, &[1, 2, 3, 4])),
            chunk(0xa020, &chunk(0x10, &f32::INFINITY.to_le_bytes().repeat(3))),
            chunk(0xa040, &chunk(0x30, &[0, 0, 0])),
            chunk(0xa050, &chunk(0x30, &f32::NAN.to_le_bytes())),
            chunk(0xa081, &[0]),
            chunk(0xa100, &[0]),
            [chunk(0xa100, &[0, 0]), chunk(0xa100, &[0, 0])].concat(),
        ] {
            assert!(Model::decode(&material_fixture(&material), ModelVariant::N).is_err());
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelVariant {
    Standard,
    N,
    X,
}

impl ModelVariant {
    pub fn from_name(name: &str) -> io::Result<Self> {
        match name.rsplit('.').next() {
            Some("3ds") => Ok(Self::Standard),
            Some("3dsN") => Ok(Self::N),
            Some("3dsX") => Ok(Self::X),
            _ => Err(invalid("unsupported model variant")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub vertices: [u16; 3],
    pub flags: u16,
    /// Nine serialized extension floats for N; semantic interpretation pending.
    pub extension: Option<[f32; 9]>,
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

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    end: usize,
}
impl Reader<'_> {
    fn take(&mut self, count: usize) -> io::Result<&[u8]> {
        let stop = self
            .position
            .checked_add(count)
            .filter(|stop| *stop <= self.end)
            .ok_or_else(|| invalid("truncated model field"))?;
        let bytes = &self.bytes[self.position..stop];
        self.position = stop;
        Ok(bytes)
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn float(&mut self) -> io::Result<f32> {
        let value = f32::from_bits(self.u32()?);
        if !value.is_finite() {
            return Err(invalid("non-finite model field"));
        }
        Ok(value)
    }
    fn floats<const N: usize>(&mut self) -> io::Result<[f32; N]> {
        let mut result = [0.0; N];
        for value in &mut result {
            *value = self.float()?;
        }
        Ok(result)
    }
    fn name(&mut self) -> io::Result<Vec<u8>> {
        let length = self.bytes[self.position..self.end]
            .iter()
            .position(|b| *b == 0)
            .ok_or_else(|| invalid("unterminated model name"))?;
        if length > 4096 {
            return Err(invalid("model name exceeds limit"));
        }
        let result = self.take(length)?.to_vec();
        self.take(1)?;
        Ok(result)
    }
    fn finish(&self) -> io::Result<()> {
        if self.position != self.end {
            return Err(invalid("trailing model field bytes"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Context {
    Main,
    Editor,
    Object(usize),
    Mesh(usize),
    Faces(usize),
    Material(usize),
    Texture(usize),
    Color(usize, u16),
    Amount(usize, u16),
}

struct Decoder<'a> {
    bytes: &'a [u8],
    model: Model,
    variant: ModelVariant,
    elements: usize,
    chunk_count: usize,
    mesh_fields: BTreeSet<(usize, u16)>,
}
impl Decoder<'_> {
    fn elements(&mut self, count: usize) -> io::Result<()> {
        self.elements += count;
        if self.elements > MAX_ELEMENTS {
            return Err(invalid("model exceeds element budget"));
        }
        Ok(())
    }
    fn walk(&mut self, start: usize, end: usize, context: Context, depth: usize) -> io::Result<()> {
        if depth > 16 {
            return Err(invalid("model chunk nesting exceeds limit"));
        }
        let mut cursor = Reader {
            bytes: self.bytes,
            position: start,
            end,
        };
        while cursor.position < end {
            self.chunk_count += 1;
            if self.chunk_count > MAX_CHUNKS {
                return Err(invalid("model chunk count exceeds limit"));
            }
            let chunk_start = cursor.position;
            let id = cursor.u16()?;
            let size = cursor.u32()? as usize;
            if size < 6 {
                return Err(invalid("model chunk smaller than header"));
            }
            let stop = chunk_start
                .checked_add(size)
                .filter(|stop| *stop <= end)
                .ok_or_else(|| invalid("model chunk exceeds parent"))?;
            let mut field = Reader {
                bytes: self.bytes,
                position: cursor.position,
                end: stop,
            };
            *self.model.chunks.entry(id).or_default() += 1;
            if let Context::Mesh(index) = context
                && matches!(id, 0x4110 | 0x4120 | 0x4140 | 0x4160)
                && !self.mesh_fields.insert((index, id))
            {
                return Err(invalid("duplicate mesh field"));
            }
            use Context::*;
            match (context, id) {
                (Main, 0x0002) => {
                    self.model.version = Some(field.u32()?);
                    field.finish()?;
                }
                (Main, 0x3d3d) => self.walk(field.position, stop, Editor, depth + 1)?,
                (Editor, 0x0100) => {
                    self.model.master_scale = Some(field.float()?);
                    field.finish()?;
                }
                (Editor, 0x4000) => {
                    self.elements(1)?;
                    let index = self.model.meshes.len();
                    self.model.meshes.push(self::Mesh {
                        name: field.name()?,
                        ..Default::default()
                    });
                    self.walk(field.position, stop, Object(index), depth + 1)?;
                }
                (Object(index), 0x4100) => {
                    self.walk(field.position, stop, Mesh(index), depth + 1)?
                }
                (Mesh(index), 0x4110) => {
                    let count = field.u16()? as usize;
                    self.elements(count)?;
                    if !self.model.meshes[index].vertices.is_empty() {
                        return Err(invalid("duplicate vertex list"));
                    }
                    for _ in 0..count {
                        self.model.meshes[index].vertices.push(field.floats()?);
                    }
                    field.finish()?;
                }
                (Mesh(index), 0x4120) => {
                    let count = field.u16()? as usize;
                    self.elements(count)?;
                    if !self.model.meshes[index].faces.is_empty() {
                        return Err(invalid("duplicate face list"));
                    }
                    for _ in 0..count {
                        let vertices = [field.u16()?, field.u16()?, field.u16()?];
                        let flags = field.u16()?;
                        let extension = if self.variant == ModelVariant::N {
                            Some(field.floats()?)
                        } else {
                            None
                        };
                        self.model.meshes[index].faces.push(Face {
                            vertices,
                            flags,
                            extension,
                        });
                    }
                    self.walk(field.position, stop, Faces(index), depth + 1)?;
                }
                (Mesh(index), 0x4140) => {
                    let count = field.u16()? as usize;
                    self.elements(count)?;
                    let mut uvs = Vec::new();
                    for _ in 0..count {
                        uvs.push(field.floats()?);
                    }
                    field.finish()?;
                    if self.model.meshes[index].uvs.replace(uvs).is_some() {
                        return Err(invalid("duplicate UV list"));
                    }
                }
                (Mesh(index), 0x4160) => {
                    let matrix = field.floats()?;
                    field.finish()?;
                    if self.model.meshes[index].matrix.replace(matrix).is_some() {
                        return Err(invalid("duplicate local matrix"));
                    }
                }
                (Faces(index), 0x4150) => {
                    let count = self.model.meshes[index].faces.len();
                    self.elements(count)?;
                    let values = (0..count)
                        .map(|_| field.u32())
                        .collect::<io::Result<Vec<_>>>()?;
                    field.finish()?;
                    if self.model.meshes[index].smoothing.replace(values).is_some() {
                        return Err(invalid("duplicate smoothing list"));
                    }
                }
                (Faces(index), 0x4130) => {
                    let name = field.name()?;
                    let count = field.u16()? as usize;
                    self.elements(count)?;
                    let faces = (0..count)
                        .map(|_| field.u16())
                        .collect::<io::Result<Vec<_>>>()?;
                    field.finish()?;
                    self.model.meshes[index]
                        .materials
                        .push(FaceMaterial { name, faces });
                }
                (Editor, 0xafff) => {
                    self.elements(1)?;
                    let index = self.model.materials.len();
                    self.model.materials.push(self::Material::default());
                    self.walk(field.position, stop, Material(index), depth + 1)?;
                }
                (Material(index), 0xa000) => {
                    let name = field.name()?;
                    field.finish()?;
                    if self.model.materials[index].name.replace(name).is_some() {
                        return Err(invalid("duplicate material name"));
                    }
                }
                (Material(index), 0xa200) => {
                    self.walk(field.position, stop, Texture(index), depth + 1)?
                }
                (Material(index), property @ (0xa010 | 0xa020 | 0xa030)) => {
                    self.walk(field.position, stop, Color(index, property), depth + 1)?;
                }
                (Color(index, property), encoding @ 0x0010..=0x0012) => {
                    let value = match encoding {
                        0x0010 => ColorValue::RgbFloat(field.floats()?),
                        0x0011 => ColorValue::Rgb8(field.take(3)?.try_into().unwrap()),
                        _ => ColorValue::LinearRgb8(field.take(3)?.try_into().unwrap()),
                    };
                    field.finish()?;
                    self.elements(1)?;
                    self.model.materials[index]
                        .colors
                        .push(ColorRecord { property, value });
                }
                (Material(index), property @ (0xa040 | 0xa041 | 0xa050 | 0xa052 | 0xa053)) => {
                    self.walk(field.position, stop, Amount(index, property), depth + 1)?;
                }
                (Amount(index, property), 0x0030) => {
                    // OpenShores extends the standard word chunk to float payloads.
                    let value = match stop - field.position {
                        2 => AmountValue::Word(field.u16()?),
                        4 if self.variant != ModelVariant::X => AmountValue::Float(field.float()?),
                        _ => return Err(invalid("unsupported material amount encoding")),
                    };
                    field.finish()?;
                    self.elements(1)?;
                    self.model.materials[index]
                        .amounts
                        .push(AmountRecord { property, value });
                }
                (Material(index), 0xa081) => {
                    field.finish()?;
                    self.model.materials[index].two_sided = true;
                }
                (Material(index), 0xa100) => {
                    let shading = field.u16()?;
                    field.finish()?;
                    if self.model.materials[index]
                        .shading
                        .replace(shading)
                        .is_some()
                    {
                        return Err(invalid("duplicate material shading"));
                    }
                }
                (Texture(index), 0xa300) => {
                    let name = field.name()?;
                    field.finish()?;
                    self.model.materials[index].textures.push(name);
                }
                _ => self.model.opaque.push(OpaqueChunk {
                    id,
                    range: chunk_start..stop,
                }),
            }
            cursor.position = stop;
        }
        Ok(())
    }
}

impl Model {
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

    pub fn decode(bytes: &[u8], variant: ModelVariant) -> io::Result<Self> {
        if bytes.len() > MAX_INPUT {
            return Err(invalid("model exceeds input limit"));
        }
        let mut root = Reader {
            bytes,
            position: 0,
            end: bytes.len(),
        };
        if root.u16()? != 0x4d4d || root.u32()? as usize != bytes.len() {
            return Err(invalid("expected exact 3DS root chunk"));
        }
        let mut decoder = Decoder {
            bytes,
            model: Self::default(),
            variant,
            elements: 0,
            chunk_count: 1,
            mesh_fields: BTreeSet::new(),
        };
        decoder.model.chunks.insert(0x4d4d, 1);
        decoder.walk(6, bytes.len(), Context::Main, 0)?;
        for mesh in &decoder.model.meshes {
            if mesh.faces.iter().any(|f| {
                f.vertices
                    .iter()
                    .any(|v| *v as usize >= mesh.vertices.len())
            }) {
                return Err(invalid("face references nonexistent vertex"));
            }
            if mesh.materials.iter().any(|group| {
                group
                    .faces
                    .iter()
                    .any(|face| *face as usize >= mesh.faces.len())
            }) {
                return Err(invalid("material references nonexistent face"));
            }
        }
        decoder.model.source = bytes.to_vec();
        Ok(decoder.model)
    }
}
