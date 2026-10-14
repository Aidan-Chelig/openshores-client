//! Base globe terrain from protocol/rng.py::AuNoise and worldgen/terrain.py.
//! Geological features, roads, vegetation, and original mesh LOD are not yet applied.
use std::{
    f64::consts::{FRAC_PI_2, PI},
    io,
};

const A: [u32; 12] = [13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59];
const B: [u32; 12] = [
    15629, 15641, 15643, 15647, 15649, 15661, 15667, 15671, 15679, 15683, 15727, 15731,
];
const C: [u32; 12] = [
    754907, 754921, 754931, 754937, 754939, 754967, 754969, 754973, 754979, 754981, 754991, 789221,
];
const D: [u32; 12] = [
    1372950023, 1372950043, 1372950049, 1372950071, 1372950077, 1372950101, 1372950133, 1372950169,
    1372950191, 1372950193, 1372950221, 1372950233,
];

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn noise_bits(seed: usize, x: u32, y: u32) -> f64 {
    let index = seed % 12;
    let pre = A[index].wrapping_mul(y).wrapping_add(x);
    let h = pre ^ pre.wrapping_shl(13);
    let value = B[index]
        .wrapping_mul(h)
        .wrapping_mul(h)
        .wrapping_add(C[index])
        .wrapping_mul(h)
        .wrapping_add(D[index])
        & 0x7fff_ffff;
    1.0 - f64::from(value) / 1_073_741_824.0
}

pub fn integer_noise(seed: usize, x: i64, y: i64) -> f64 {
    noise_bits(seed, x as u32, y as u32)
}

fn scurve(t: f64) -> f64 {
    t * t * 3.0 - 2.0 * t.powf(3.0)
}

fn coherent_noise(seed: usize, x: f64, y: f64) -> f64 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;
    // Python's integer lattice is unbounded, then masked to 32 bits.
    let xb = ix.rem_euclid(4_294_967_296.0) as u32;
    let yb = iy.rem_euclid(4_294_967_296.0) as u32;
    let v00 = noise_bits(seed, xb, yb);
    let v10 = noise_bits(seed, xb.wrapping_add(1), yb);
    let v01 = noise_bits(seed, xb, yb.wrapping_add(1));
    let v11 = noise_bits(seed, xb.wrapping_add(1), yb.wrapping_add(1));
    let sx = (1.0 - (scurve(fx) * PI).cos()) * 0.5;
    let sy = (1.0 - (scurve(fy) * PI).cos()) * 0.5;
    let top = v10 * sx + v00 * (1.0 - sx);
    let bottom = v01 * (1.0 - sx) + v11 * sx;
    top * (1.0 - sy) + bottom * sy
}

fn perlin(persistence: f64, x: f64, y: f64) -> f64 {
    (0..7)
        .map(|i| {
            coherent_noise(i, 2.0f64.powi(i as i32) * x, 2.0f64.powi(i as i32) * y)
                * persistence.powf(i as f64)
        })
        .sum()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainParameters {
    /// Amplitude, detail, persistence, noise X/Y offsets, relief (wire f32).
    values: [f32; 6],
}

impl TerrainParameters {
    pub fn new(values: [f32; 6]) -> io::Result<Self> {
        if values.iter().any(|v| !v.is_finite())
            || values[0] < 0.0
            || values[1] <= 0.0
            || !(0.0..=1.0).contains(&values[2])
            || !(-1.0..=1.0).contains(&values[5])
        {
            return Err(invalid("invalid base terrain parameters"));
        }
        Ok(Self { values })
    }

    pub fn values(&self) -> [f32; 6] {
        self.values
    }

    /// Latitude and longitude are radians. Returns reference base altitude MSL.
    pub fn altitude_msl(&self, size: u8, lat: f64, lon: f64) -> io::Result<f64> {
        if !lat.is_finite()
            || !lon.is_finite()
            || !(-FRAC_PI_2..=FRAC_PI_2).contains(&lat)
            || !(-PI..=PI).contains(&lon)
        {
            return Err(invalid(
                "terrain coordinates outside latitude/longitude range",
            ));
        }
        let [amp, detail, bias, noise_x, noise_y, relief] = self.values.map(f64::from);
        let x = if (lon - PI).abs() <= 0.00001 {
            -PI
        } else {
            lon
        };
        let cos_scale = lat.cos() * 0.999999 + 0.000001;
        let sample = |longitude: f64| {
            perlin(
                bias,
                (longitude / PI) * cos_scale * amp + noise_x,
                (lat / PI) * amp + noise_y,
            ) / detail
                - relief
        };
        let mut h = sample(x);
        const WIDTH: f64 = 0.17453292519943295;
        if x > 2.9670597283903604 {
            let wrapped = sample(x - 2.0 * PI);
            let blend = (1.0 - (((PI - x) / WIDTH) * PI).cos()) * 0.5;
            h = (1.0 - blend) * wrapped + blend * h;
        }
        if lat > 1.3962634015954636 {
            let pole = perlin(bias, noise_x, amp * 0.5 + noise_y) / detail - relief;
            let t = (FRAC_PI_2 - lat) / WIDTH;
            h = pole * (1.0 - t) + t * h;
        } else if lat < -1.3962634015954636 {
            let pole = perlin(bias, noise_x, noise_y - amp * 0.5) / detail - relief;
            let t = (lat + FRAC_PI_2) / WIDTH;
            h = pole * (1.0 - t) + t * h;
        }
        let shaped = if relief == 1.0 || (h <= 0.0 && relief != -1.0) {
            h /= relief + 1.0;
            -(-h).powf((h + 2.0).max(1.0))
        } else {
            h /= 1.0 - relief;
            if h > 0.0 && h < 1.0 {
                h *= h;
            }
            h
        };
        let altitude = (if size > 2 { 1500.0 } else { 1000.0 }) * shaped;
        if !altitude.is_finite() {
            return Err(invalid("terrain sample is non-finite"));
        }
        Ok(altitude)
    }

    /// The reference uses Z as the globe's polar axis.
    pub fn surface_position(&self, size: u8, lat: f64, lon: f64) -> io::Result<[f64; 3]> {
        let radius = globe_radius(size) + self.altitude_msl(size, lat, lon)?;
        Ok([
            radius * lat.cos() * lon.cos(),
            radius * lat.cos() * lon.sin(),
            radius * lat.sin(),
        ])
    }
}

pub fn globe_radius(size: u8) -> f64 {
    f64::from(size) * (1000.0 / 1.609344 * 5.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlobeSurface {
    pub size: u8,
    pub atmosphere_type: u8,
    pub atmosphere_density: u8,
    pub water: u8,
    pub terrain: TerrainParameters,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_python_altitude_fixtures() {
        for line in include_str!("../tests/fixtures/terrain.tsv")
            .lines()
            .filter(|line| !line.starts_with('#'))
        {
            let parts: Vec<_> = line.split_whitespace().collect();
            let size = parts[0].parse().unwrap();
            let values = std::array::from_fn(|i| parts[i + 1].parse().unwrap());
            let terrain = TerrainParameters::new(values).unwrap();
            let lat = parts[7].parse().unwrap();
            let lon = parts[8].parse().unwrap();
            let expected: f64 = parts[9].parse().unwrap();
            let actual = terrain.altitude_msl(size, lat, lon).unwrap();
            assert!(
                (actual - expected).abs() <= 1e-8,
                "{line}: {actual} != {expected}"
            );
        }
    }

    #[test]
    fn seams_poles_and_invalid_inputs() {
        let terrain = TerrainParameters::new([3.0, 2.26, 0.65, 123.37, 456.81, 0.0]).unwrap();
        for lat in [-1.0, 0.0, 1.0] {
            assert_eq!(
                terrain.altitude_msl(8, lat, -PI).unwrap(),
                terrain.altitude_msl(8, lat, PI).unwrap()
            );
        }
        for lat in [-FRAC_PI_2, FRAC_PI_2] {
            assert!(
                (terrain.altitude_msl(8, lat, 0.0).unwrap()
                    - terrain.altitude_msl(8, lat, 2.0).unwrap())
                .abs()
                    < 1e-8
            );
        }
        assert!(terrain.altitude_msl(8, 2.0, 0.0).is_err());
        assert!(terrain.altitude_msl(8, 0.0, f64::NAN).is_err());
        assert!(TerrainParameters::new([3.0, 0.0, 0.65, 0.0, 0.0, 0.0]).is_err());
        let radius = globe_radius(8);
        assert!((radius - 24854.84768949336).abs() < 1e-8);
    }
}
