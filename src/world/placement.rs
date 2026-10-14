//! Source-profile local transforms relative to an explicit ancestor.
use super::{WorldState, invalid};
use crate::atom_position::{PackedPosition, ParentEnvironment, compose_environment};
use std::io;

impl WorldState {
    /// Compose known raw local fields up to (excluding) the requested ancestor.
    /// None means a missing node, missing local transform, or unrelated ancestor.
    /// This is not native packet dispatch, environment lookup or interpolation.
    pub fn transform_relative_to(
        &self,
        id: u32,
        ancestor: u32,
    ) -> io::Result<Option<[[f64; 4]; 4]>> {
        if !self.entities.contains_key(&ancestor) {
            return Ok(None);
        }
        let mut current = id;
        let mut matrix =
            std::array::from_fn(|column| std::array::from_fn(|row| f64::from(column == row)));
        // Public entity maps can be edited directly, bypassing update cycle checks.
        for _ in 0..=self.entities.len() {
            let Some(entity) = self.entities.get(&current) else {
                return Ok(None);
            };
            if current == ancestor {
                return Ok(Some(matrix));
            }
            let (Some(parent), Some(local)) = (entity.parent, entity.transform) else {
                return Ok(None);
            };
            let local = PackedPosition {
                flags: 0,
                parent: None,
                position: local.position.map(f64::from),
                rotation: local.rotation,
            }
            .local_matrix()?;
            matrix = compose_environment(matrix, ParentEnvironment::Matrix(local))?;
            current = parent;
        }
        Err(invalid("cycle in relative transform ancestry"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{AtomKind, Entity, Transform};

    fn node(id: u32, parent: Option<u32>, position: [f32; 3], rotation: [f32; 3]) -> Entity {
        Entity {
            id,
            parent,
            transform: Some(Transform { position, rotation }),
            kind: AtomKind::Item,
            timestamp_ms: 0,
            created_ms: None,
            modified_ms: None,
            name: None,
            world_identity: None,
            globe_surface: None,
            geology: None,
            person: None,
            item: None,
            last_body: vec![],
            last_base_flags: 0,
        }
    }

    #[test]
    fn ancestor_is_excluded_and_parent_rotation_affects_child_translation() {
        let mut world = WorldState::default();
        world
            .entities
            .insert(1, node(1, None, [999.0; 3], [0.0; 3]));
        world.entities.insert(
            2,
            node(
                2,
                Some(1),
                [10.0, 20.0, 30.0],
                [0.0, 0.0, std::f32::consts::FRAC_PI_2],
            ),
        );
        world
            .entities
            .insert(3, node(3, Some(2), [2.0, 3.0, 4.0], [0.0; 3]));
        let matrix = world.transform_relative_to(3, 1).unwrap().unwrap();
        for (actual, expected) in matrix[3].into_iter().zip([7.0, 22.0, 34.0, 1.0]) {
            assert!((actual - expected).abs() < 1e-6);
        }
        assert_eq!(
            world.transform_relative_to(3, 2).unwrap().unwrap()[3],
            [2.0, 3.0, 4.0, 1.0]
        );
        assert_eq!(
            world.transform_relative_to(1, 1).unwrap().unwrap()[3],
            [0.0, 0.0, 0.0, 1.0]
        );
        assert!(world.transform_relative_to(1, 3).unwrap().is_none());
        world.entities.remove(&2);
        assert!(world.transform_relative_to(3, 1).unwrap().is_none());
    }

    #[test]
    fn direct_map_cycles_and_missing_transforms_are_not_guessed() {
        let mut world = WorldState::default();
        world.entities.insert(1, node(1, None, [0.0; 3], [0.0; 3]));
        world
            .entities
            .insert(2, node(2, Some(2), [0.0; 3], [0.0; 3]));
        assert!(world.transform_relative_to(2, 1).is_err());
        world.entities.get_mut(&2).unwrap().parent = Some(1);
        world.entities.get_mut(&2).unwrap().transform = None;
        assert!(world.transform_relative_to(2, 1).unwrap().is_none());
        assert!(world.transform_relative_to(2, 99).unwrap().is_none());
    }
}
