//! CPU body deformation, starting with ApplyScaleFactor's unweighted branch.
//! This is not whole-avatar generation or renderer/world-coordinate conversion.
use super::{Mesh, ModelVariant, NativeBodyObjectGroups};
use std::io;

/// Actual object fields consumed by the no-matching-group path. Values come
/// from native body model definitions/metadata, not a renderer display scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeBodyObjectScale {
    pub field_140: f64,
    pub fields_1c0: [f32; 3],
    pub fields_1f4: [f32; 3],
}

impl NativeBodyObjectScale {
    /// ApplyScaleFactor's proven no-group/no-matching-group branch. A caller
    /// handling groups must establish non-membership before using this method.
    /// Native computes two products before subtraction, not (point-origin)*s.
    /// Zero scale skips all point/origin arithmetic, including nonfinite input.
    pub fn apply_unweighted_vertex(&self, [x, y, z]: [f64; 3]) -> [f64; 3] {
        let [mut ox, mut oy, mut oz] = self.fields_1c0.map(f64::from);
        let s = self.field_140;
        let [sx, sy, sz] = if s == 0. {
            [0.; 3]
        } else {
            let [cx, cy, cz] = self.fields_1f4.map(f64::from);
            let sx = x * s;
            let sy = y * s;
            ox -= cx * s;
            oy -= cy * s;
            let sz = s * z;
            oz -= cz * s;
            [sx, sy, sz]
        };
        [ox + sx, oy + sy, sz + oz]
    }
}

impl Mesh {
    /// Native loaded coordinates followed by the unweighted CPU body scale.
    /// Point-only groups are extracted as native object metadata first. Any
    /// retained deformation groups remain unsupported, never a uniform fallback.
    pub fn body_unweighted_positions(
        &self,
        variant: ModelVariant,
        loader_scale: f64,
        definition: &NativeBodyObjectScale,
    ) -> io::Result<Vec<[f64; 3]>> {
        let groups = NativeBodyObjectGroups::from_mesh(self)?;
        if !groups.retained_groups().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native body mesh requires weighted group deformation",
            ));
        }
        let mut definition = *definition;
        if let Some(connection) = groups.control_point(0x1cc) {
            definition.fields_1f4 = connection;
        }
        Ok(self
            .loaded_positions(variant, loader_scale)?
            .into_iter()
            .map(|point| definition.apply_unweighted_vertex(point))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::VertexGroup;

    #[test]
    #[allow(clippy::if_same_then_else)] // Native Z has opposite operand order.
    fn actual_float_metadata_is_widened_before_independent_double_products() {
        let scale = NativeBodyObjectScale {
            field_140: 1.234567890123,
            fields_1c0: [0.1, -0.2, 0.3],
            fields_1f4: [-0.7, 0.8, -0.9],
        };
        let input = [1.234567891, -2.345678912, 3.456789123];
        let before = scale;
        let actual = scale.apply_unweighted_vertex(input);
        let expected = std::array::from_fn(|i| {
            let offset =
                f64::from(scale.fields_1c0[i]) - f64::from(scale.fields_1f4[i]) * scale.field_140;
            if i == 2 {
                scale.field_140 * input[i] + offset
            } else {
                offset + input[i] * scale.field_140
            }
        });
        assert_eq!(actual, expected);
        assert_eq!(scale, before);
    }

    #[test]
    fn cancellation_does_not_rewrite_to_subtract_before_multiply() {
        let scale = NativeBodyObjectScale {
            field_140: 0.1,
            fields_1c0: [0.; 3],
            fields_1f4: [1.; 3],
        };
        let x = f64::from_bits(1_f64.to_bits() + 1);
        let actual = scale.apply_unweighted_vertex([x; 3]);
        let rearranged = (x - 1.) * scale.field_140;
        assert_ne!(actual[0].to_bits(), rearranged.to_bits());
        assert_eq!(actual[0], x * scale.field_140 - scale.field_140);
    }

    #[test]
    fn zero_scale_skips_nonfinite_inputs_but_nonzero_nan_scales_do_not() {
        for s in [0., -0.] {
            let scale = NativeBodyObjectScale {
                field_140: s,
                fields_1c0: [1., -2., 3.],
                fields_1f4: [f32::NAN; 3],
            };
            assert_eq!(
                scale.apply_unweighted_vertex([f64::NAN, f64::INFINITY, f64::NEG_INFINITY]),
                [1., -2., 3.]
            );
        }
        for s in [1., f64::NAN, f64::INFINITY] {
            let scale = NativeBodyObjectScale {
                field_140: s,
                fields_1c0: [1.; 3],
                fields_1f4: [f32::NAN; 3],
            };
            assert!(
                scale
                    .apply_unweighted_vertex([0.; 3])
                    .iter()
                    .all(|v| v.is_nan())
            );
        }
    }

    #[test]
    fn mesh_coordinates_are_loaded_before_body_deformation_without_source_mutation() {
        let mesh = Mesh {
            vertices: vec![[1., 2., 3.], [-1., -2., -3.]],
            ..Default::default()
        };
        let definition = NativeBodyObjectScale {
            field_140: 2.,
            fields_1c0: [0.5, 1., 1.5],
            fields_1f4: [0.25, 0.5, 0.75],
        };
        let loaded = mesh.loaded_positions(ModelVariant::N, 0.5).unwrap();
        let actual = mesh
            .body_unweighted_positions(ModelVariant::N, 0.5, &definition)
            .unwrap();
        assert_eq!(
            actual,
            loaded
                .into_iter()
                .map(|p| definition.apply_unweighted_vertex(p))
                .collect::<Vec<_>>()
        );
        assert_eq!(mesh.vertices, [[1., 2., 3.], [-1., -2., -3.]]);
    }

    #[test]
    fn any_unresolved_group_is_unsupported_instead_of_successful_wrong_geometry() {
        let mesh = Mesh {
            vertex_groups: vec![VertexGroup::default()],
            ..Default::default()
        };
        let definition = NativeBodyObjectScale {
            field_140: 1.,
            fields_1c0: [0.; 3],
            fields_1f4: [0.; 3],
        };
        assert_eq!(
            mesh.body_unweighted_positions(ModelVariant::N, 1., &definition)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
}
