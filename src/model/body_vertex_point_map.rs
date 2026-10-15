//! Native vertex->AuPoint hash identity, including the stored hash comparison.
use super::NativeBodyStoredVertex;
use std::{collections::BTreeMap, io};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativeBodyVertexPointMap(BTreeMap<(u32, [u32; 3]), [f64; 3]>);

fn key(vertex: &NativeBodyStoredVertex) -> io::Result<(u32, [u32; 3])> {
    let [x, y, z] = vertex.position;
    let a = f32::from_bits(vertex.payload.words_00_through_24[8]);
    let b = f32::from_bits(vertex.payload.words_00_through_24[9]);
    let truncate = |value: f32| {
        let value = f64::from(value);
        if !value.is_finite()
            || !(-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&value)
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native vertex hash nonfinite/out-of-range conversion",
            ));
        }
        Ok(value as i64 as u32)
    };
    let hash = truncate((x + y) + z)? ^ truncate(a + b)?;
    if vertex.position.iter().any(|v| v.is_nan()) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native vertex hash NaN equality",
        ));
    }
    Ok((
        hash,
        vertex
            .position
            .map(|v| if v == 0. { 0 } else { v.to_bits() }),
    ))
}

impl NativeBodyVertexPointMap {
    /// Emplace does not overwrite: hash equality plus xyz float equality are
    /// required. Bone/weight payload is ignored;20/24 affect the hash only.
    pub fn insert_first(
        &mut self,
        vertex: &NativeBodyStoredVertex,
        point: [f64; 3],
    ) -> io::Result<()> {
        self.0.entry(key(vertex)?).or_insert(point);
        Ok(())
    }
    pub fn get(&self, vertex: &NativeBodyStoredVertex) -> io::Result<Option<[f64; 3]>> {
        Ok(self.0.get(&key(vertex)?).copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NativeBodyVertexWeights;
    #[test]
    fn native_point_map_matches_hash_and_xyz_not_full_payload() {
        let mut map = NativeBodyVertexPointMap::default();
        let mut vertex = NativeBodyStoredVertex {
            position: [0.; 3],
            payload: NativeBodyVertexWeights::default(),
        };
        map.insert_first(&vertex, [1.; 3]).unwrap();
        vertex.position = [-0.; 3];
        vertex.payload.words_00_through_24[0] = 60;
        vertex.payload.words_00_through_24[8] = 0.5_f32.to_bits();
        assert_eq!(map.get(&vertex).unwrap(), Some([1.; 3]));
        map.insert_first(&vertex, [2.; 3]).unwrap();
        assert_eq!(map.get(&vertex).unwrap(), Some([1.; 3]));
        vertex.payload.words_00_through_24[8] = 1_f32.to_bits();
        assert_eq!(map.get(&vertex).unwrap(), None);
        vertex.payload.words_00_through_24[8] = f32::NAN.to_bits();
        assert_eq!(
            map.get(&vertex).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
}
