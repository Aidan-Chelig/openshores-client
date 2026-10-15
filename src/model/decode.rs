use super::*;
use std::collections::BTreeSet;

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
    Group(usize, usize),
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
                    self.model.meshes[index]
                        .geometry_order
                        .push(GeometryChunk::Vertices);
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
                        let extension =
                            if matches!(self.variant, ModelVariant::N | ModelVariant::NT) {
                                Some(field.floats()?)
                            } else {
                                None
                            };
                        let nt_extension = if self.variant == ModelVariant::NT {
                            Some(field.floats()?)
                        } else {
                            None
                        };
                        self.model.meshes[index].faces.push(Face {
                            vertices,
                            flags,
                            extension,
                            nt_extension,
                        });
                    }
                    self.walk(field.position, stop, Faces(index), depth + 1)?;
                }
                (Mesh(index), 0x4112) => {
                    self.elements(1)?;
                    let group = self.model.meshes[index].vertex_groups.len();
                    self.model.meshes[index]
                        .vertex_groups
                        .push(VertexGroup::default());
                    self.walk(field.position, stop, Group(index, group), depth + 1)?;
                }
                (Group(index, group), 0x4113) => {
                    let name = field.name()?;
                    field.finish()?;
                    if self.model.meshes[index].vertex_groups[group]
                        .name
                        .replace(name)
                        .is_some()
                    {
                        return Err(invalid("duplicate vertex group name"));
                    }
                    self.model.meshes[index].vertex_groups[group]
                        .field_order
                        .push(VertexGroupChunk::Name);
                }
                (Group(index, group), 0x4114) => {
                    let count = usize::from(field.u16()?);
                    self.elements(count)?;
                    let mut members = Vec::with_capacity(count);
                    for _ in 0..count {
                        members.push(GroupVertex {
                            position: field.floats()?,
                            weight: field.float()?,
                        });
                    }
                    field.finish()?;
                    if self.model.meshes[index].vertex_groups[group]
                        .members
                        .replace(members)
                        .is_some()
                    {
                        return Err(invalid("duplicate vertex group members"));
                    }
                    self.model.meshes[index].vertex_groups[group]
                        .field_order
                        .push(VertexGroupChunk::Members);
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
                    self.model.meshes[index]
                        .geometry_order
                        .push(GeometryChunk::MappingCoordinates);
                }
                (Mesh(index), 0x4160) => {
                    let matrix = field.floats()?;
                    field.finish()?;
                    if self.model.meshes[index].matrix.replace(matrix).is_some() {
                        return Err(invalid("duplicate local matrix"));
                    }
                    self.model.meshes[index]
                        .geometry_order
                        .push(GeometryChunk::Matrix);
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
