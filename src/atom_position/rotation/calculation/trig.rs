/// Native expanded inverse-XY rotation. Keep exact trig branches and operation
/// order (including signed-zero products), rather than simplifying to a matrix.
pub(super) fn z_angle(right: [f64; 3], x: f64, y: f64, has_x: bool, down: bool) -> f64 {
    let (sx, cx) = if has_x { x.sin_cos() } else { (0.0, 1.0) };
    let (sy, cy) = if down {
        (1.2246467991473532e-16, -1.0)
    } else {
        y.sin_cos()
    };
    let (sx, cx, zero_cx, zero_sx) = if cx.abs() == 1.0 {
        (0.0, cx, cx * 0.0, 0.0)
    } else if sx.abs() == 1.0 {
        (sx, 0.0, 0.0, sx * 0.0)
    } else {
        (sx, cx, cx * 0.0, sx * 0.0)
    };
    let (xx, xz, yx, xy, yy, yz) = if cy.abs() == 1.0 {
        (cy, zero_cx, -cy * 0.0, zero_sx, zero_sx, zero_cx)
    } else {
        let (xx, yx) = if sy.abs() == 1.0 {
            (0.0, -0.0)
        } else {
            (cy, -cy * 0.0)
        };
        (xx, sy * cx, yx, sy * sx, (sx * sy) * 0.0, (sy * 0.0) * cx)
    };
    let projected_y = ((cx - yy) * right[1] + yx * right[0]) + (sx + yz) * right[2];
    let projected_x = ((zero_cx + xy) * right[1] + xx * right[0]) + (zero_sx - xz) * right[2];
    projected_y.atan2(projected_x)
}
