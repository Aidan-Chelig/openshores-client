//! Post-face torso source-key deformation and same-kind destination map updates.
use super::NativeBodyMergeGeometry;
use std::io;

impl NativeBodyMergeGeometry {
    /// Run AFTER source vertex/face/corner processing. Native rebuilds source
    /// kind7/9 key maps, then transfers maps into EXISTING destination groups
    /// by kind, not name. Kind9 replaces, all others insert/overwrite entries.
    /// Other group metadata/control points/flags are not copied or invented.
    /// Returns count of same-kind destination/source map update operations.
    pub fn transfer_torso_group_maps(&mut self, source: &mut Self) -> io::Result<usize> {
        for index in 0..source.object.groups.retained_groups().len() {
            let group = &source.object.groups.retained_groups()[index];
            if !matches!(group.kind_34, 7 | 9) {
                continue;
            }
            let updated = group.remap_positions(|point| {
                let point = point.map(f64::from);
                let selected = source.object.groups.select_vertex(
                    point,
                    source.scale_points[0],
                    source.scale_points[1],
                )?;
                let output = source
                    .deformation
                    .apply_vertex(point, &selected)?
                    .map(|v| v as f32);
                if output.iter().any(|v| !v.is_finite()) {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "nonfinite torso source group coordinate",
                    ));
                }
                Ok(output)
            })?;
            source.object.groups.replace_retained_group(index, updated);
        }
        let mut writes = 0;
        for index in 0..self.object.groups.retained_groups().len() {
            let mut destination = self.object.groups.retained_groups()[index].clone();
            for source in source.object.groups.retained_groups() {
                if source.kind_34 == destination.kind_34 {
                    destination.merge_weight_map_from(source, destination.kind_34 == 9);
                    writes += 1;
                }
            }
            self.object
                .groups
                .replace_retained_group(index, destination);
        }
        Ok(writes)
    }
}

#[cfg(test)]
mod tests;
