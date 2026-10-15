use super::*;

fn dna(exo: bool, wrist: bool) -> NativeDhDna {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[16] = u8::from(!exo);
    dna.0[11] = if wrist { 0 } else { 0x80 };
    dna
}
fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-6, "{a} != {b}");
}

#[test]
fn endoskeletal_equilateral_solution_and_wrist_compensation() {
    let a = calculate(&dna(false, true), 1., 0., 1., 1., 0.25).unwrap();
    near(a.shoulder, -HALF - std::f32::consts::FRAC_PI_3);
    near(a.elbow, PI - std::f32::consts::FRAC_PI_3);
    near(a.wrist, 2. * std::f32::consts::FRAC_PI_3 - HALF);
    let b = calculate(&dna(false, false), 1., 0., 1., 0.75, 0.25).unwrap();
    assert_eq!(a.shoulder, b.shoulder);
    assert_eq!(a.elbow, b.elbow);
    assert_eq!(b.wrist, f32::from_bits(0x3d0efa35));
}

#[test]
fn projected_origin_is_added_before_narrowing_and_reach_is_capped() {
    let a = calculate(&dna(false, true), 0.75, 0.25, 1., 1., 0.5).unwrap();
    assert_eq!(
        a,
        calculate(&dna(false, true), 1., 0., 1., 1., 0.5).unwrap()
    );
    let a = calculate(&dna(false, true), 100., 0., 1., 1., 0.5).unwrap();
    assert_eq!(
        a,
        calculate(&dna(false, true), 200., 0., 1., 1., 0.5).unwrap()
    );
    assert_eq!(a.wrist, f32::from_bits(0x3d0efa35));
}

#[test]
fn exoskeletal_fallback_overwrites_invalid_intermediate_triangle_angles() {
    // Initial acos denominators contain zero, but native overwrites both.
    let a = calculate(&dna(true, true), 0., 0., 1., 1., 0.5).unwrap();
    near(a.shoulder, -HALF - f32::from_bits(0x4016cbe4));
    near(
        a.elbow,
        PI - ((f32::from_bits(0x3f3504f3)).acos() + f32::from_bits(0x3f490fdb)),
    );
    // Native wrist compensation uses both RAW angles before output conversion.
    near(a.wrist, f32::from_bits(0x4016cbe4));
    let b = calculate(&dna(true, true), 1., 0., 1., 1., 0.5).unwrap();
    assert_eq!(b.shoulder, -PI);
    assert_eq!(b.elbow, HALF);
    assert_eq!(b.wrist, HALF);
}

#[test]
fn invalid_final_domains_and_nonfinite_inputs_are_not_clamped() {
    for h in [f32::NAN, f32::INFINITY, -10.] {
        assert_eq!(
            calculate(&dna(true, true), h, 0., 1., 1., 0.5)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
    assert_eq!(
        calculate(&dna(false, true), 1., f64::NAN, 1., 1., 0.5)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
}
