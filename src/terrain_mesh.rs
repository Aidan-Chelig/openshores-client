//! Diagnostic base-terrain tessellation, not reconstructed OpenShores mesh LOD.
use crate::terrain::{GlobeSurface, globe_radius};
use std::{
    f64::consts::{FRAC_PI_2, PI},
    io::{self, Write},
};

/// Synthetic globe used by the Python parity fixture, not a game-world default.
pub fn demonstration_surface() -> GlobeSurface {
    GlobeSurface {
        size: 8,
        atmosphere_type: 0,
        atmosphere_density: 60,
        water: 40,
        terrain: crate::terrain::TerrainParameters::new([3.0, 2.26, 0.65, 123.37, 456.81, 0.0])
            .expect("fixed validated demonstration parameters"),
    }
}

#[derive(Debug)]
pub struct TerrainMesh {
    positions: Vec<[f64; 3]>,
    triangles: Vec<[u32; 3]>,
    statistics: TerrainStatistics,
}

/// Extrema over generated vertices only, not continuous/global terrain extrema.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainStatistics {
    pub latitude_bands: u32,
    pub longitude_sectors: u32,
    pub base_radius: f64,
    pub minimum_altitude_msl: f64,
    pub maximum_altitude_msl: f64,
    pub vertices: usize,
    pub triangles: usize,
}

impl TerrainMesh {
    /// Uniform latitude bands, twice as many longitude sectors, shared seam and
    /// single pole vertices. Resolution is an inspector choice, not game LOD.
    pub fn generate(surface: GlobeSurface, bands: u32) -> io::Result<Self> {
        if !(3..=128).contains(&bands) || surface.size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "terrain mesh requires 3..=128 latitude bands and nonzero globe size",
            ));
        }
        let sectors = bands * 2;
        let mut minimum_altitude = f64::INFINITY;
        let mut maximum_altitude = f64::NEG_INFINITY;
        let mut positions = Vec::with_capacity((2 + (bands - 1) * sectors) as usize);
        for row in 0..=bands {
            let lat = -FRAC_PI_2 + PI * f64::from(row) / f64::from(bands);
            let count = if row == 0 || row == bands { 1 } else { sectors };
            for column in 0..count {
                let lon = if count == 1 {
                    0.0
                } else {
                    -PI + 2.0 * PI * f64::from(column) / f64::from(sectors)
                };
                let altitude = surface.terrain.altitude_msl(surface.size, lat, lon)?;
                minimum_altitude = minimum_altitude.min(altitude);
                maximum_altitude = maximum_altitude.max(altitude);
                let radius = globe_radius(surface.size) + altitude;
                if !radius.is_finite() || radius <= 0.0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "terrain mesh requires finite positive surface radii",
                    ));
                }
                let point = if row == 0 {
                    [0.0, 0.0, -radius]
                } else if row == bands {
                    [0.0, 0.0, radius]
                } else {
                    [
                        radius * lat.cos() * lon.cos(),
                        radius * lat.cos() * lon.sin(),
                        radius * lat.sin(),
                    ]
                };
                if point.iter().any(|v| !v.is_finite()) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "non-finite terrain mesh vertex",
                    ));
                }
                positions.push(point);
            }
        }
        let ring = |row: u32, column: u32| 1 + (row - 1) * sectors + column % sectors;
        let mut triangles = Vec::with_capacity((2 * sectors * (bands - 1)) as usize);
        for column in 0..sectors {
            triangles.push([0, ring(1, column + 1), ring(1, column)]);
            for row in 1..bands - 1 {
                let [a, b, c, d] = [
                    ring(row, column),
                    ring(row, column + 1),
                    ring(row + 1, column),
                    ring(row + 1, column + 1),
                ];
                triangles.extend([[a, b, c], [b, d, c]]);
            }
            triangles.push([
                ring(bands - 1, column),
                ring(bands - 1, column + 1),
                positions.len() as u32 - 1,
            ]);
        }
        Ok(Self {
            statistics: TerrainStatistics {
                latitude_bands: bands,
                longitude_sectors: sectors,
                base_radius: globe_radius(surface.size),
                minimum_altitude_msl: minimum_altitude,
                maximum_altitude_msl: maximum_altitude,
                vertices: positions.len(),
                triangles: triangles.len(),
            },
            positions,
            triangles,
        })
    }

    pub fn positions(&self) -> &[[f64; 3]] {
        &self.positions
    }
    pub fn statistics(&self) -> TerrainStatistics {
        self.statistics
    }
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }

    pub fn write_obj(&self, output: &mut impl Write) -> io::Result<()> {
        writeln!(
            output,
            "# OpenShores diagnostic base terrain; raw globe-local Z-up coordinates"
        )?;
        writeln!(
            output,
            "# No geology, water surface, textures, original LOD or scene transforms"
        )?;
        writeln!(output, "o base_terrain")?;
        for [x, y, z] in &self.positions {
            writeln!(output, "v {x} {y} {z}")?;
        }
        for face in &self.triangles {
            let [a, b, c] = face.map(|v| v + 1);
            writeln!(output, "f {a} {b} {c}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::TerrainParameters;
    use std::collections::BTreeMap;

    fn surface() -> GlobeSurface {
        GlobeSurface {
            size: 3,
            atmosphere_type: 0,
            atmosphere_density: 0,
            water: 0,
            terrain: TerrainParameters::new([1.0, 1.0, 0.5, 0.0, 0.0, 0.0]).unwrap(),
        }
    }

    #[test]
    fn mesh_is_closed_with_outward_winding_and_exact_samples() {
        for bands in [3, 8, 32] {
            let mesh = TerrainMesh::generate(surface(), bands).unwrap();
            let sectors = bands * 2;
            assert_eq!(mesh.positions.len(), (2 + (bands - 1) * sectors) as usize);
            assert_eq!(mesh.triangles.len(), (2 * sectors * (bands - 1)) as usize);
            let mut edges = BTreeMap::new();
            for &[a, b, c] in &mesh.triangles {
                assert!(a != b && b != c && c != a);
                for (u, v) in [(a, b), (b, c), (c, a)] {
                    let key = (u.min(v), u.max(v));
                    let entry = edges.entry(key).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += if u < v { 1 } else { -1 };
                }
                let [a, b, c] = [a, b, c].map(|i| mesh.positions[i as usize]);
                let u: [f64; 3] = std::array::from_fn(|i| b[i] - a[i]);
                let v: [f64; 3] = std::array::from_fn(|i| c[i] - a[i]);
                let n = [
                    u[1] * v[2] - u[2] * v[1],
                    u[2] * v[0] - u[0] * v[2],
                    u[0] * v[1] - u[1] * v[0],
                ];
                assert!((0..3).map(|i| n[i] * a[i]).sum::<f64>() > 0.0);
            }
            assert!(
                edges
                    .values()
                    .all(|&(count, balance)| count == 2 && balance == 0)
            );
            assert_eq!(
                mesh.positions.len() as i64 - edges.len() as i64 + mesh.triangles.len() as i64,
                2
            );
            for row in 1..bands {
                for column in 0..sectors {
                    let lat = -FRAC_PI_2 + PI * f64::from(row) / f64::from(bands);
                    let lon = -PI + 2.0 * PI * f64::from(column) / f64::from(sectors);
                    assert_eq!(
                        mesh.positions[(1 + (row - 1) * sectors + column) as usize],
                        surface()
                            .terrain
                            .surface_position(surface().size, lat, lon)
                            .unwrap()
                    );
                }
            }
        }
    }

    #[test]
    fn statistics_report_sampled_altitudes_and_counts() {
        let surface = demonstration_surface();
        let mesh = TerrainMesh::generate(surface, 16).unwrap();
        let stats = mesh.statistics();
        assert_eq!(stats.latitude_bands, 16);
        assert_eq!(stats.longitude_sectors, 32);
        assert_eq!(stats.vertices, mesh.positions().len());
        assert_eq!(stats.triangles, mesh.triangles().len());
        assert_eq!(stats.base_radius, globe_radius(surface.size));
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for row in 0..=16 {
            let lat = -FRAC_PI_2 + PI * row as f64 / 16.0;
            for column in 0..if row == 0 || row == 16 { 1 } else { 32 } {
                let lon = if row == 0 || row == 16 {
                    0.0
                } else {
                    -PI + 2.0 * PI * column as f64 / 32.0
                };
                let altitude = surface
                    .terrain
                    .altitude_msl(surface.size, lat, lon)
                    .unwrap();
                low = low.min(altitude);
                high = high.max(altitude);
            }
        }
        assert_eq!(stats.minimum_altitude_msl, low);
        assert_eq!(stats.maximum_altitude_msl, high);
    }

    #[test]
    fn demo_matches_reference_globe_fixture() {
        let text = include_str!("../tests/fixtures/atom-globe.hex");
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let packet: Vec<u8> = (0..compact.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&compact[i..i + 2], 16).unwrap())
            .collect();
        let mut world = crate::world::WorldState::default();
        world.ingest(&packet).unwrap();
        assert_eq!(
            world.entities[&7].globe_surface,
            Some(demonstration_surface())
        );
        assert!(TerrainMesh::generate(demonstration_surface(), 32).is_ok());
    }

    #[test]
    fn bounded_generation_and_obj_errors() {
        for bands in [0, 2, 129, u32::MAX] {
            assert!(TerrainMesh::generate(surface(), bands).is_err());
        }
        let mut invalid = surface();
        invalid.size = 0;
        assert!(TerrainMesh::generate(invalid, 8).is_err());
        invalid = surface();
        invalid.terrain =
            TerrainParameters::new([1.0, f32::MIN_POSITIVE, 0.5, 0.0, 0.0, 0.0]).unwrap();
        assert!(TerrainMesh::generate(invalid, 8).is_err());
        let mesh = TerrainMesh::generate(surface(), 3).unwrap();
        let mut output = Vec::new();
        mesh.write_obj(&mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert_eq!(
            text.lines().filter(|line| line.starts_with("v ")).count(),
            mesh.positions.len()
        );
        assert_eq!(
            text.lines().filter(|line| line.starts_with("f ")).count(),
            mesh.triangles.len()
        );
        assert!(text.contains("f 1 3 2\n"));
        struct Fails;
        impl Write for Fails {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("test"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        assert!(mesh.write_obj(&mut Fails).is_err());
    }
}
