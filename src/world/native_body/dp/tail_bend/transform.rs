use crate::world::native_body::NativeDpBoneTransform;

// Expanded in-place RotateX then RotateZ coefficients, including fourth row.
pub(crate) struct Bend {
    a: f64,
    b: f64,
    d: f64,
    e: f64,
    f: f64,
    g: f64,
    h: f64,
    i: f64,
    j: f64,
    k: f64,
    l: f64,
    ground: bool,
}
impl Bend {
    pub(crate) fn new(pitch: f32, yaw: f32) -> Self {
        let (s, c) = f64::from(pitch).sin_cos();
        let cz = c * 0.;
        let sz = s * 0.;
        let sum = cz + sz;
        let (z, w) = f64::from(yaw).sin_cos();
        let wz = w * 0.;
        let nz = -z * 0.;
        Self {
            a: w + z * sum,
            b: sum * w - z,
            d: cz - sz,
            e: z * (c + sz) + wz,
            f: (c + sz) * w + nz,
            g: cz - s,
            h: z * (s + cz) + wz,
            i: w * (s + cz) + nz,
            j: c - sz,
            k: z * sum + wz,
            l: sum * w + nz,
            ground: false,
        }
    }
    pub(super) fn ground() -> Self {
        Self {
            a: 1.,
            b: 0.,
            d: 0.,
            e: 0.,
            f: 0.9996573249564867,
            g: 0.026176949036146674,
            h: 0.,
            i: -0.026176949036146674,
            j: 0.9996573249564867,
            k: 0.,
            l: 0.,
            ground: true,
        }
    }
    pub(crate) fn about_slither(&self, [x, y, z]: [f64; 3]) -> [f64; 4] {
        let xz = x * 0.;
        let yz = y * 0.;
        let zz = z * 0.;
        [
            ((((((yz + x) + zz) + 0.) + self.a * -x) + self.b * -y) + self.d * -z),
            ((((((-y * self.f + self.e * -x) + self.g * -z) + xz) + y) + zz) + 0.),
            ((((self.j * -z + self.i * -y) + self.h * -x) + z) + (yz + xz)) + 0.,
            (((((zz + (yz + xz)) + 1.) + -x * self.k) + -y * self.l) + self.d * -z),
        ]
    }
    pub(crate) fn apply_slither(&self, bone: &mut NativeDpBoneTransform, t: [f64; 4]) {
        let m = bone.matrix_00;
        for col in 0..4 {
            let k = col * 4;
            let [x, y, z, w] = [m[k], m[k + 1], m[k + 2], m[k + 3]];
            bone.matrix_00[k] = ((x * self.a + y * self.b) + z * self.d) + w * t[0];
            bone.matrix_00[k + 1] = ((self.g * z + y * self.f) + self.e * x) + t[1] * w;
            bone.matrix_00[k + 2] = if col == 0 {
                ((self.i * y + self.h * x) + self.j * z) + t[2] * w
            } else {
                ((z * self.j + self.i * y) + self.h * x) + t[2] * w
            };
            bone.matrix_00[k + 3] = ((x * self.k + y * self.l) + self.d * z) + w * t[3];
        }
        bone.field_100 = 0;
    }
    pub(super) fn about(&self, [x, y, z]: [f64; 3]) -> [f64; 4] {
        let xz = x * 0.;
        let yz = y * 0.;
        let zz = z * 0.;
        if self.ground {
            return [
                (((((-yz - x) - zz) + x) + yz) + zz) + 0.,
                (((((-z * self.g + (-y * self.f - xz)) + y) + xz) + zz) + 0.),
                (((((-z * self.j + (y * self.g - xz)) + z) + yz) + xz) + 0.),
                (((((-xz - yz) - zz) + zz) + yz) + xz) + 1.,
            ];
        }
        [
            (((((self.a * -x + self.b * -y) + self.d * -z) + yz) + x) + zz) + 0.,
            (((((self.g * -z + self.f * -y) + self.e * -x) + xz) + y) + zz) + 0.,
            ((((self.j * -z + self.i * -y) + self.h * -x) + z) + (yz + xz)) + 0.,
            ((((self.k * -x + self.l * -y) + self.d * -z) + zz) + (yz + xz)) + 1.,
        ]
    }
    pub(super) fn apply(&self, bone: &mut NativeDpBoneTransform, t: [f64; 4]) {
        if self.ground {
            self.apply_ground(bone, t);
            return;
        }
        let m = bone.matrix_00;
        for col in 0..4 {
            let k = col * 4;
            let [x, y, z, w] = [m[k], m[k + 1], m[k + 2], m[k + 3]];
            bone.matrix_00[k] = ((self.a * x + self.b * y) + z * self.d) + t[0] * w;
            bone.matrix_00[k + 1] = ((self.g * z + self.f * y) + self.e * x) + t[1] * w;
            bone.matrix_00[k + 2] = if col == 2 {
                ((self.i * y + self.h * x) + self.j * z) + t[2] * w
            } else {
                ((self.j * z + self.i * y) + self.h * x) + t[2] * w
            };
            bone.matrix_00[k + 3] = ((x * self.k + y * self.l) + z * self.d) + w * t[3];
        }
        bone.field_100 = 0;
    }

    fn apply_ground(&self, bone: &mut NativeDpBoneTransform, t: [f64; 4]) {
        let m = bone.matrix_00;
        for col in 0..4 {
            let k = col * 4;
            let [x, y, z, w] = [m[k], m[k + 1], m[k + 2], m[k + 3]];
            bone.matrix_00[k] = if col < 2 {
                ((z * 0. + x) + y * 0.) + t[0] * w
            } else {
                ((y * 0. + x) + z * 0.) + t[0] * w
            };
            bone.matrix_00[k + 1] = if col < 2 {
                ((z * self.g + y * self.f) + x * 0.) + t[1] * w
            } else {
                ((x * 0. + y * self.f) + z * self.g) + t[1] * w
            };
            bone.matrix_00[k + 2] = if col < 2 {
                ((z * self.j + x * 0.) + y * -self.g) + w * t[2]
            } else {
                ((w * t[2] + z * self.j) + y * -self.g) + x * 0.
            };
            bone.matrix_00[k + 3] = if col < 2 {
                ((z * 0. + y * 0.) + x * 0.) + w * t[3]
            } else {
                ((w * t[3] + z * 0.) + x * 0.) + y * 0.
            };
        }
        bone.field_100 = 0;
    }
}

fn terms(bone: &NativeDpBoneTransform, [x, y, z]: [f64; 3]) -> ([f64; 4], [f64; 4], [f64; 4]) {
    let m = &bone.matrix_00;
    (
        std::array::from_fn(|i| if x == 0. { 0. } else { x * m[i] }),
        std::array::from_fn(|i| if y == 0. { 0. } else { m[4 + i] * y }),
        std::array::from_fn(|i| if z == 0. { 0. } else { z * m[8 + i] }),
    )
}
pub(super) fn project(bone: &NativeDpBoneTransform, point: [f64; 3]) -> [f64; 3] {
    let m = &bone.matrix_00;
    let (x, y, z) = terms(bone, point);
    let w = if point.iter().all(|v| *v == 0.) {
        ((m[3] + m[7]) + m[11]) + m[15]
    } else {
        ((x[3] + y[3]) + z[3]) + m[15]
    };
    let mut p = if point.iter().all(|v| *v == 0.) {
        [m[12], m[13], m[14]]
    } else {
        [
            ((m[12] + y[0]) + x[0]) + z[0],
            ((m[13] + x[1]) + y[1]) + z[1],
            ((m[14] + x[2]) + y[2]) + z[2],
        ]
    };
    if w != 0. && w != 1. {
        for v in &mut p {
            *v /= w;
        }
    }
    p
}
// Native unrolled paths reverse addition operands; retain their expressions.
#[allow(clippy::if_same_then_else)]
pub(super) fn project_z(bone: &NativeDpBoneTransform, point: [f64; 3], ordinal: usize) -> f64 {
    // The ground test's inlined ProjectZ uses its own summation order.
    let m = &bone.matrix_00;
    let (x, y, z) = terms(bone, point);
    let mut value = if point[0] == 0. && point[2] == 0. {
        if point[1] == 0. {
            m[14]
        } else {
            m[14] + (y[2] + 0.)
        }
    } else {
        ((z[2] + y[2]) + x[2]) + m[14]
    };
    let w = if point.iter().all(|v| *v == 0.) {
        if ordinal == 4 {
            ((m[3] + m[7]) + m[11]) + m[15]
        } else {
            m[15] + ((m[7] + m[3]) + m[11])
        }
    } else if point[0] == 0. && point[2] == 0. {
        if ordinal == 4 {
            (y[3] + 0.) + m[15]
        } else {
            m[15] + (y[3] + 0.)
        }
    } else if point[0] == 0. || (point[2] != 0. && ordinal == 0) {
        ((z[3] + y[3]) + x[3]) + m[15]
    } else {
        ((y[3] + x[3]) + z[3]) + m[15]
    };
    if w != 0. && w != 1. {
        value /= w;
    }
    value
}
