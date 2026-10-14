//! Geometry-only debugging exports; deliberately not the OpenShores renderer.
use crate::model::Model;
use std::io::{self, Write};

const MAX_ELEMENTS: usize = 1_000_000;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub minimum: [f64; 3],
    pub maximum: [f64; 3],
}

/// Validate before emitting any bytes, including callers' manually built models.
pub fn bounds(model: &Model) -> io::Result<Option<Bounds>> {
    let mut result: Option<Bounds> = None;
    let mut count = 0usize;
    for mesh in &model.meshes {
        count = count
            .checked_add(1 + mesh.vertices.len() + mesh.faces.len())
            .filter(|count| *count <= MAX_ELEMENTS)
            .ok_or_else(|| invalid("preview exceeds geometry budget"))?;
        for vertex in &mesh.vertices {
            if vertex.iter().any(|value| !value.is_finite()) {
                return Err(invalid("non-finite preview vertex"));
            }
            let point = vertex.map(f64::from);
            match result.as_mut() {
                None => {
                    result = Some(Bounds {
                        minimum: point,
                        maximum: point,
                    })
                }
                Some(bounds) => {
                    for (axis, value) in point.iter().enumerate() {
                        bounds.minimum[axis] = bounds.minimum[axis].min(*value);
                        bounds.maximum[axis] = bounds.maximum[axis].max(*value);
                    }
                }
            }
        }
        if mesh.faces.iter().any(|face| {
            face.vertices
                .iter()
                .any(|index| *index as usize >= mesh.vertices.len())
        }) {
            return Err(invalid("invalid preview face index"));
        }
    }
    Ok(result)
}

/// Vertex coordinates, face winding and mesh separation are preserved.
/// No normals, UVs, materials, matrices, master scale, or animation are applied.
pub fn write_obj(model: &Model, output: &mut impl Write) -> io::Result<()> {
    bounds(model)?;
    writeln!(output, "# OpenShores raw geometry debugging export")?;
    writeln!(
        output,
        "# Unconverted coordinates; transforms, UVs, normals, materials and animation omitted"
    )?;
    let mut offset = 1usize;
    for (index, mesh) in model.meshes.iter().enumerate() {
        // Generated identifiers prevent source names from injecting OBJ directives.
        writeln!(output, "o mesh_{index}")?;
        for vertex in &mesh.vertices {
            writeln!(output, "v {} {} {}", vertex[0], vertex[1], vertex[2])?;
        }
        for face in &mesh.faces {
            let indices = face.vertices.map(|vertex| offset + vertex as usize);
            writeln!(output, "f {} {} {}", indices[0], indices[1], indices[2])?;
        }
        offset += mesh.vertices.len();
    }
    Ok(())
}

/// Three orthographic raw-axis views, fit independently with uniform scaling.
/// No depth sorting/occlusion, lighting, coordinate conversion or source shaders.
pub fn write_svg(model: &Model, output: &mut impl Write) -> io::Result<()> {
    let bounds = bounds(model)?;
    let vertices: usize = model.meshes.iter().map(|mesh| mesh.vertices.len()).sum();
    let faces: usize = model.meshes.iter().map(|mesh| mesh.faces.len()).sum();
    writeln!(
        output,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="1260" height="490" viewBox="0 0 1260 490">"#
    )?;
    writeln!(output, "<title>OpenShores raw geometry inspection</title>")?;
    writeln!(
        output,
        "<desc>Three orthographic wireframe projections. Not the original renderer. Transforms, materials, normals, UVs and animation are omitted.</desc>"
    )?;
    writeln!(
        output,
        r##"<rect width="1260" height="490" fill="#101820"/>"##
    )?;
    writeln!(
        output,
        r##"<g font-family="monospace" fill="#e0edf5" font-size="14">"##
    )?;
    writeln!(
        output,
        r#"<text x="20" y="24">RAW GEOMETRY DEBUG VIEW — {} meshes, {vertices} vertices, {faces} faces</text>"#,
        model.meshes.len()
    )?;
    writeln!(
        output,
        r#"<text x="20" y="475">No source transforms, materials, normals, UVs or animation. Not an in-game preview.</text></g>"#
    )?;
    for (panel, (horizontal, vertical, label)) in
        [(0, 1, "X / Y"), (0, 2, "X / Z"), (1, 2, "Y / Z")]
            .into_iter()
            .enumerate()
    {
        let left = panel as f64 * 420.0;
        writeln!(
            output,
            r##"<rect x="{}" y="38" width="400" height="410" fill="none" stroke="#354553"/>"##,
            left + 10.0
        )?;
        writeln!(
            output,
            r##"<text x="{}" y="60" font-family="monospace" font-size="14" fill="#e0edf5">{label}</text>"##,
            left + 25.0
        )?;
        let Some(bounds) = bounds else {
            continue;
        };
        let span = (bounds.maximum[horizontal] - bounds.minimum[horizontal])
            .max(bounds.maximum[vertical] - bounds.minimum[vertical]);
        let scale = if span == 0.0 { 1.0 } else { 340.0 / span };
        let center_x = (bounds.minimum[horizontal] + bounds.maximum[horizontal]) * 0.5;
        let center_y = (bounds.minimum[vertical] + bounds.maximum[vertical]) * 0.5;
        let project = |point: [f32; 3]| {
            (
                left + 210.0 + (f64::from(point[horizontal]) - center_x) * scale,
                255.0 - (f64::from(point[vertical]) - center_y) * scale,
            )
        };
        writeln!(
            output,
            r##"<g fill="none" stroke="#80c8e5" stroke-width="0.55" stroke-opacity="0.55">"##
        )?;
        for mesh in &model.meshes {
            for face in &mesh.faces {
                let points = face
                    .vertices
                    .map(|index| project(mesh.vertices[index as usize]));
                writeln!(
                    output,
                    r#"<path d="M {:.3} {:.3} L {:.3} {:.3} L {:.3} {:.3} Z"/>"#,
                    points[0].0, points[0].1, points[1].0, points[1].1, points[2].0, points[2].1
                )?;
            }
        }
        writeln!(output, "</g>")?;
    }
    writeln!(output, "</svg>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Face, Mesh};

    fn triangle() -> Mesh {
        Mesh {
            name: b"unsafe\nf 999 999 999\n<script>".to_vec(),
            vertices: vec![[-0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            faces: vec![Face {
                vertices: [0, 1, 2],
                flags: 0,
                extension: None,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn obj_preserves_coordinates_winding_and_global_indices() {
        let model = Model {
            meshes: vec![triangle(), triangle()],
            ..Default::default()
        };
        let mut bytes = Vec::new();
        write_obj(&model, &mut bytes).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("v -0 0 0\n"));
        assert!(text.contains("f 1 2 3\n"));
        assert!(text.contains("f 4 5 6\n"));
        assert_eq!(
            text.lines().filter(|line| line.starts_with("o ")).count(),
            2
        );
        assert!(!text.contains("999"));
        assert!(!text.contains("vn "));
        assert!(!text.contains("vt "));
    }

    #[test]
    fn svg_is_finite_and_omits_untrusted_names() {
        let mut mesh = triangle();
        mesh.vertices = vec![
            [-f32::MAX, 0.0, 0.0],
            [f32::MAX, 0.0, 0.0],
            [0.0, f32::MAX, 0.0],
        ];
        for vertices in [mesh.vertices.clone(), vec![[1.0, 1.0, 1.0]; 3]] {
            mesh.vertices = vertices;
            let model = Model {
                meshes: vec![mesh],
                ..Default::default()
            };
            let mut bytes = Vec::new();
            write_svg(&model, &mut bytes).unwrap();
            let text = String::from_utf8(bytes).unwrap();
            assert_eq!(text.matches("<path ").count(), 3);
            assert!(!text.contains("NaN"));
            assert!(!text.contains("inf"));
            assert!(!text.contains("<script>"));
            assert!(text.ends_with("</svg>\n"));
            mesh = triangle();
        }
        let mut bytes = Vec::new();
        write_svg(&Model::default(), &mut bytes).unwrap();
        assert!(!String::from_utf8(bytes).unwrap().contains("<path "));
    }

    #[test]
    fn invalid_models_and_write_failures_propagate() {
        let mut mesh = triangle();
        mesh.faces[0].vertices[2] = 3;
        let mut model = Model {
            meshes: vec![mesh],
            ..Default::default()
        };
        let mut bytes = Vec::new();
        assert!(write_obj(&model, &mut bytes).is_err());
        assert!(bytes.is_empty());
        assert!(write_svg(&model, &mut bytes).is_err());
        assert!(bytes.is_empty());
        model.meshes[0] = triangle();
        model.meshes[0].vertices[0][0] = f32::NAN;
        assert!(write_svg(&model, &mut bytes).is_err());
        assert!(bytes.is_empty());
        let model = Model {
            meshes: vec![triangle()],
            ..Default::default()
        };
        let mut too_small = &mut [0u8; 4][..];
        assert!(write_obj(&model, &mut too_small).is_err());
    }
}
