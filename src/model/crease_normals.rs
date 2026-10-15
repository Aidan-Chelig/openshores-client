//! Modern angle-based corner normals, independent of renderer and native data.
//! Exact-position seams share adjacency; native vertex/index arrays stay intact.
use std::{collections::BTreeMap, io};

fn root(parents: &mut [usize], mut i: usize) -> usize {
    while parents[i] != i {
        parents[i] = parents[parents[i]];
        i = parents[i];
    }
    i
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn normalized(a: [f64; 3]) -> Option<[f64; 3]> {
    let length = dot(a, a).sqrt();
    (length > 0. && length.is_finite()).then(|| a.map(|v| v / length))
}

/// Area-weighted normals in face-major corner order. Smoothing crosses only
/// manifold edges with angle STRICTLY below the threshold, never vertex-only
/// contact. Degenerate faces receive diagnostic +Z, matching the part viewer;
/// this is an explicit modern preview policy, not a native normal reconstruction.
pub fn crease_corner_normals(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    degrees: f64,
) -> io::Result<Vec<[f32; 3]>> {
    if !degrees.is_finite()
        || !(0.0..=180.).contains(&degrees)
        || positions.iter().flatten().any(|v| !v.is_finite())
        || triangles
            .iter()
            .flatten()
            .any(|i| *i as usize >= positions.len())
        || triangles.len().checked_mul(3).is_none()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid crease-normal geometry or angle",
        ));
    }
    let mut unique = BTreeMap::new();
    let ids: Vec<_> = positions
        .iter()
        .map(|point| {
            let key = point.map(|v| if v == 0. { 0 } else { v.to_bits() });
            let next = unique.len();
            *unique.entry(key).or_insert(next)
        })
        .collect();
    let normals: Vec<_> = triangles
        .iter()
        .map(|face| {
            let [a, b, c] = face.map(|i| positions[i as usize].map(f64::from));
            let u: [f64; 3] = std::array::from_fn(|i| b[i] - a[i]);
            let v: [f64; 3] = std::array::from_fn(|i| c[i] - a[i]);
            [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ]
        })
        .collect();
    let mut edges: BTreeMap<[usize; 2], Vec<[usize; 2]>> = BTreeMap::new();
    for (f, face) in triangles.iter().enumerate() {
        if normalized(normals[f]).is_none() {
            continue;
        }
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            let (a, b) = if ids[face[a] as usize] < ids[face[b] as usize] {
                (a, b)
            } else {
                (b, a)
            };
            edges
                .entry([ids[face[a] as usize], ids[face[b] as usize]])
                .or_default()
                .push([f * 3 + a, f * 3 + b]);
        }
    }
    let mut parents: Vec<_> = (0..triangles.len() * 3).collect();
    let cosine = degrees.to_radians().cos();
    for adjacency in edges.values() {
        let [a, b] = adjacency.as_slice() else {
            continue;
        };
        if dot(
            normalized(normals[a[0] / 3]).unwrap(),
            normalized(normals[b[0] / 3]).unwrap(),
        )
        .clamp(-1., 1.)
            > cosine
        {
            for endpoint in 0..2 {
                let x = root(&mut parents, a[endpoint]);
                let y = root(&mut parents, b[endpoint]);
                parents[x] = y;
            }
        }
    }
    let mut sums = vec![[0.; 3]; parents.len()];
    for corner in 0..parents.len() {
        let group = root(&mut parents, corner);
        for axis in 0..3 {
            sums[group][axis] += normals[corner / 3][axis];
        }
    }
    Ok((0..parents.len())
        .map(|corner| {
            let group = root(&mut parents, corner);
            normalized(sums[group])
                .unwrap_or([0., 0., 1.])
                .map(|v| v as f32)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_inputs_without_panicking() {
        assert!(crease_corner_normals(&[], &[[0; 3]], 60.).is_err());
        assert!(crease_corner_normals(&[[f32::NAN; 3]], &[], 60.).is_err());
        for angle in [f64::NAN, -1., 181.] {
            assert!(crease_corner_normals(&[], &[], angle).is_err());
        }
        assert!(crease_corner_normals(&[], &[], 60.).unwrap().is_empty());
    }
    #[test]
    fn duplicate_seams_join_but_threshold_and_nonmanifold_stay_hard() {
        let p = [
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            [-0., 0., 0.],
            [1., 0., 0.],
            [0., 0., 1.],
            [0., -1., 0.],
        ];
        let faces = [[0, 1, 2], [3, 5, 4]];
        let hard = crease_corner_normals(&p, &faces, 90.).unwrap();
        assert_eq!(hard[0], [0., 0., 1.]);
        assert_eq!(hard[3], [0., 1., 0.]);
        let smooth = crease_corner_normals(&p, &faces, 91.).unwrap();
        assert_eq!(smooth[0], smooth[3]);
        assert!((smooth[0][1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
        let split = crease_corner_normals(&p, &[[0, 1, 2], [3, 5, 4], [0, 1, 6]], 180.).unwrap();
        assert_eq!(split[0], [0., 0., 1.]);
        assert_eq!(split[3], [0., 1., 0.]);
    }
}
