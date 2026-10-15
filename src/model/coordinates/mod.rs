//! Native mesh-coordinate stages, separate from raw decoding and world placement.
//!
//! See docs/native-models.md for shipped-artifact evidence and limitations.
//! Helper filtering, materials, animation, entity axes/rotation are not modeled.
use super::{GeometryChunk, Mesh, ModelVariant, invalid};
use std::io;

mod matrix;
#[cfg(test)]
mod tests;
use matrix::Matrix;

fn narrow(point: [f64; 3]) -> io::Result<[f32; 3]> {
    let point = point.map(|value| value as f32);
    if point.iter().any(|value| !value.is_finite()) {
        return Err(invalid("non-finite native mesh coordinate"));
    }
    Ok(point)
}

fn into_local(points: &mut [[f32; 3]], inverse: &Matrix) -> io::Result<()> {
    for point in points {
        *point = narrow(inverse.project(point.map(f64::from)))?;
    }
    Ok(())
}

impl Mesh {
    /// Reconstruct coordinates emitted by the native triangle builder.
    /// `loader_scale` is its positive caller parameter, NOT the 3DS master scale
    /// or a renderer's world-display scale. Raw data is never mutated.
    /// Singular matrices are rejected instead of emitting native partial-inverse
    /// results. This method does not filter metadata/helper meshes.
    pub fn loaded_positions(
        &self,
        variant: ModelVariant,
        loader_scale: f64,
    ) -> io::Result<Vec<[f64; 3]>> {
        let (points, matrix) = self.loaded_vertex_state(variant, loader_scale)?;
        points
            .into_iter()
            .map(|point| {
                let point = matrix.project(point.map(f64::from));
                if point.iter().any(|value| !value.is_finite()) {
                    Err(invalid("non-finite projected mesh coordinate"))
                } else {
                    Ok(point)
                }
            })
            .collect()
    }

    /// Stored native object xyz after ordered geometry chunks, before later
    /// loader-scale and triangle-builder projection/deformation stages.
    pub fn native_weight_lookup_positions(
        &self,
        variant: ModelVariant,
    ) -> io::Result<Vec<[f32; 3]>> {
        self.loaded_vertex_state(variant, 1.)
            .map(|(points, _)| points)
    }

    fn loaded_vertex_state(
        &self,
        variant: ModelVariant,
        loader_scale: f64,
    ) -> io::Result<(Vec<[f32; 3]>, Matrix)> {
        if !loader_scale.is_finite() || loader_scale <= 0.0 {
            return Err(invalid("loader scale must be finite and positive"));
        }
        let order = if self.geometry_order.is_empty() && self.matrix.is_none() {
            // A manually constructed matrix-free mesh has no order ambiguity.
            &[GeometryChunk::Vertices][..]
        } else {
            &self.geometry_order
        };
        if order.len() > 2
            || order
                .iter()
                .filter(|chunk| **chunk == GeometryChunk::Vertices)
                .count()
                > 1
            || order
                .iter()
                .filter(|chunk| **chunk == GeometryChunk::Matrix)
                .count()
                != usize::from(self.matrix.is_some())
            || (!self.vertices.is_empty() && !order.contains(&GeometryChunk::Vertices))
        {
            return Err(invalid("missing or inconsistent geometry chunk order"));
        }
        let mut points = Vec::new();
        let mut matrix = Matrix::identity();
        let mut inverse = None;
        for chunk in order {
            match chunk {
                GeometryChunk::Vertices => {
                    points = self.vertices.clone();
                    if points.iter().flatten().any(|value| !value.is_finite()) {
                        return Err(invalid("non-finite raw mesh coordinate"));
                    }
                    if let Some(inverse) = &inverse {
                        into_local(&mut points, inverse)?;
                    }
                }
                GeometryChunk::Matrix => {
                    let factor = if variant == ModelVariant::X {
                        1.0 / 12.0
                    } else {
                        1.0
                    };
                    if factor != 1.0 {
                        for point in &mut points {
                            *point = narrow(point.map(|value| f64::from(value) * factor))?;
                        }
                    }
                    matrix = Matrix::from_serialized(self.matrix.unwrap(), factor)?;
                    if !matrix.is_identity() {
                        inverse = Some(matrix.inverse()?);
                        into_local(&mut points, inverse.as_ref().unwrap())?;
                    }
                }
            }
        }
        if loader_scale != 1.0 {
            matrix.scale_translation(loader_scale);
            for point in &mut points {
                *point = narrow(point.map(|value| f64::from(value) * loader_scale))?;
            }
        }
        Ok((points, matrix))
    }
}
