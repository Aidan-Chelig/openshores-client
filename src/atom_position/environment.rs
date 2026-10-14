//! DaAtom::EnvTransform composition branches, without inferred class dispatch.
use std::io;

/// Established result of resolving the native parent and its virtual predicate.
/// Do not infer Boundary from a globe's name or from an unresolved atom kind.
#[derive(Clone, Debug, PartialEq)]
pub enum ParentEnvironment {
    /// Native Parent() returned null; the environment matrix becomes identity.
    Missing,
    /// Parent CanRenderEnvironment() returned true; keep the local matrix.
    Boundary,
    /// Parent predicate returned false; compose its EnvTransform with local.
    Matrix([[f64; 4]; 4]),
}

/// Column-major matrices: parent environment * local. Does not choose a parent,
/// resolve virtual class dispatch, cache results, or mutate a world/scene.
/// Non-finite inputs/results are rejected as a Rust safety boundary.
pub fn compose_environment(
    local: [[f64; 4]; 4],
    parent: ParentEnvironment,
) -> io::Result<[[f64; 4]; 4]> {
    if parent == ParentEnvironment::Missing {
        return Ok(std::array::from_fn(|column| {
            std::array::from_fn(|row| f64::from(column == row))
        }));
    }
    let result = match parent {
        ParentEnvironment::Missing => unreachable!(),
        ParentEnvironment::Boundary => local,
        ParentEnvironment::Matrix(parent) => {
            if parent.iter().flatten().any(|value| !value.is_finite()) {
                return Err(invalid());
            }
            std::array::from_fn(|column| {
                std::array::from_fn(|row| (0..4).map(|k| parent[k][row] * local[column][k]).sum())
            })
        }
    };
    if local
        .iter()
        .flatten()
        .chain(result.iter().flatten())
        .any(|value| !value.is_finite())
    {
        return Err(invalid());
    }
    Ok(result)
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "non-finite environment matrix")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_parent_boundary_and_rotated_parent_have_distinct_results() {
        let identity = compose_environment([[f64::NAN; 4]; 4], ParentEnvironment::Missing).unwrap();
        let mut local = identity;
        local[3] = [2.0, 3.0, 4.0, 1.0];
        assert_eq!(
            compose_environment(local, ParentEnvironment::Boundary).unwrap(),
            local
        );
        let parent = [
            [0.0, 1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [10.0, 20.0, 30.0, 1.0],
        ];
        let matrix = compose_environment(local, ParentEnvironment::Matrix(parent)).unwrap();
        assert_eq!(matrix[3], [7.0, 22.0, 34.0, 1.0]);
        assert_eq!(matrix[..3], parent[..3]);
        assert_eq!(
            compose_environment(local, ParentEnvironment::Missing).unwrap(),
            identity
        );
    }

    #[test]
    fn finite_safety_applies_to_used_matrices_and_composition_overflow() {
        assert!(compose_environment([[f64::NAN; 4]; 4], ParentEnvironment::Boundary).is_err());
        assert!(
            compose_environment(
                [[1.0; 4]; 4],
                ParentEnvironment::Matrix([[f64::INFINITY; 4]; 4])
            )
            .is_err()
        );
        assert!(
            compose_environment([[f64::MAX; 4]; 4], ParentEnvironment::Matrix([[2.0; 4]; 4]))
                .is_err()
        );
    }
}
