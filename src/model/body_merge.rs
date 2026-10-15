//! Native stored-vertex operation required by head merging, not a full merge.
use super::NativeBodyVertexWeights;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeBodyStoredVertex {
    pub payload: NativeBodyVertexWeights,
    pub position: [f32; 3],
}

/// Au3dsGLSLVertexArray::AddIfUnique. Ordered first-match search, strict
/// Euclidean tolerance, f64 differences of stored f32 values. If requested,
/// compare fields20/24 as floats (not bit identities). A nearby vertex with
/// unequal fields is appended with its position snapped to the first match;
/// its entire other payload is retained. False also means snap-and-append.
pub fn native_body_add_if_unique(
    vertices: &mut Vec<NativeBodyStoredVertex>,
    mut vertex: NativeBodyStoredVertex,
    tolerance: f64,
    compare_fields_20_24: bool,
) -> usize {
    for (index, existing) in vertices.iter().enumerate() {
        let delta = std::array::from_fn::<_, 3, _>(|i| {
            f64::from(vertex.position[i]) - f64::from(existing.position[i])
        });
        let distance = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
        if distance < tolerance {
            let equal = [8, 9].into_iter().all(|i| {
                f32::from_bits(existing.payload.words_00_through_24[i])
                    == f32::from_bits(vertex.payload.words_00_through_24[i])
            });
            if compare_fields_20_24 && equal {
                return index;
            }
            vertex.position = existing.position;
            break;
        }
    }
    let index = vertices.len();
    vertices.push(vertex);
    index
}

#[cfg(test)]
mod tests {
    use super::*;
    fn vertex(x: f32, field: f32) -> NativeBodyStoredVertex {
        let mut payload = NativeBodyVertexWeights::default();
        payload.words_00_through_24[8] = field.to_bits();
        NativeBodyStoredVertex {
            position: [x, 0., 0.],
            payload,
        }
    }
    #[test]
    fn native_vertex_uniqueness_preserves_strict_boundary_and_first_match() {
        let mut vertices = vec![vertex(0., 0.)];
        assert_eq!(
            native_body_add_if_unique(&mut vertices, vertex(1., 0.), 1., true),
            1
        );
        assert_eq!(
            native_body_add_if_unique(&mut vertices, vertex(0.5, 0.), 1., true),
            0
        );
        assert_eq!(vertices.len(), 2);
        assert_eq!(
            native_body_add_if_unique(&mut vertices, vertex(0.5, 2.), 1., true),
            2
        );
        assert_eq!(vertices[2].position, [0.; 3]);
        assert_eq!(vertices[2].payload.words_00_through_24[8], 2_f32.to_bits());
        assert_eq!(
            native_body_add_if_unique(&mut vertices, vertex(0.5, 0.), 1., false),
            3
        );
        assert_eq!(vertices[3].position, [0.; 3]);
    }
    #[test]
    fn native_vertex_uniqueness_uses_float_equality_not_bits() {
        let mut vertices = vec![vertex(0., -0.)];
        assert_eq!(
            native_body_add_if_unique(&mut vertices, vertex(0., 0.), 1., true),
            0
        );
        vertices[0] = vertex(0., f32::NAN);
        assert_eq!(
            native_body_add_if_unique(&mut vertices, vertex(0., f32::NAN), 1., true),
            1
        );
        assert_eq!(
            native_body_add_if_unique(&mut vertices, vertex(0., 0.), 0., true),
            2
        );
    }
}
