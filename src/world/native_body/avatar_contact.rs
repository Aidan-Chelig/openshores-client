//! Render-only intersection decisions; not native animation or collision code.
pub(super) type Triangle = [[f64; 3]; 3];
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn edges(t: Triangle) -> [[f64; 3]; 3] {
    [sub(t[1], t[0]), sub(t[2], t[1]), sub(t[0], t[2])]
}
fn range(t: Triangle, axis: [f64; 3]) -> [f64; 2] {
    let p = t.map(|v| dot(v, axis));
    [
        p.into_iter().fold(f64::INFINITY, f64::min),
        p.into_iter().fold(f64::NEG_INFINITY, f64::max),
    ]
}
fn overlaps(a: [f64; 2], b: [f64; 2]) -> bool {
    a[0] <= b[1] && b[0] <= a[1]
}
pub(super) fn has_area(t: Triangle) -> bool {
    let e = edges(t);
    let n = cross(e[0], e[1]);
    dot(n, n) > 0.
}
fn bounds(t: Triangle) -> [[f64; 2]; 3] {
    std::array::from_fn(|i| range(t, std::array::from_fn(|j| if i == j { 1. } else { 0. })))
}

/// SAT for nondegenerate triangles, including in-plane axes for coplanar faces.
/// Zero-area triangles produce no rendered pixels and are not contact witnesses.
fn touch(a: Triangle, b: Triangle) -> bool {
    let ea = edges(a);
    let eb = edges(b);
    let na = cross(ea[0], ea[1]);
    let nb = cross(eb[0], eb[1]);
    if dot(na, na) == 0. || dot(nb, nb) == 0. {
        return false;
    }
    let separates = |axis| !overlaps(range(a, axis), range(b, axis));
    if separates(na) || separates(nb) {
        return false;
    }
    for edge in ea {
        if separates(cross(na, edge)) {
            return false;
        }
        for other in eb {
            if separates(cross(edge, other)) {
                return false;
            }
        }
    }
    for edge in eb {
        if separates(cross(nb, edge)) {
            return false;
        }
    }
    true
}

pub(super) fn meshes_touch(a: &[Triangle], b: &[Triangle]) -> bool {
    let bounded: Vec<_> = b.iter().map(|t| (*t, bounds(*t))).collect();
    a.iter().any(|a| {
        let ba = bounds(*a);
        bounded
            .iter()
            .any(|(b, bb)| (0..3).all(|i| overlaps(ba[i], bb[i])) && touch(*a, *b))
    })
}

pub(super) fn point_triangle_distance(p: [f64; 3], [a, b, c]: Triangle) -> f64 {
    let n = cross(sub(b, a), sub(c, a));
    let n2 = dot(n, n);
    if n2 > 0. {
        let height = dot(sub(p, a), n) / n2;
        let q = std::array::from_fn(|i| p[i] - n[i] * height);
        if [(a, b), (b, c), (c, a)]
            .into_iter()
            .all(|(a, b)| dot(cross(sub(b, a), sub(q, a)), n) >= 0.)
        {
            return (height * height * n2).sqrt();
        }
    }
    [(a, b), (b, c), (c, a)]
        .into_iter()
        .map(|(a, b)| {
            let d = sub(b, a);
            let d2 = dot(d, d);
            let t = if d2 == 0. {
                0.
            } else {
                (dot(sub(p, a), d) / d2).clamp(0., 1.)
            };
            let v = std::array::from_fn(|i| p[i] - (a[i] + d[i] * t));
            dot(v, v).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contact_includes_coplanar_and_piercing_but_not_separation_or_zero_area() {
        let a = [[0., 0., 0.], [2., 0., 0.], [0., 2., 0.]];
        assert!(touch(a, a));
        assert!(touch(a, [[0.5, 0.5, -1.], [0.5, 0.5, 1.], [3., 3., 1.]]));
        assert!(touch(a, [[0.5, -1., 0.], [0.5, 2., 0.], [1.5, 2., 0.]]));
        assert!(!touch(a, a.map(|p| [p[0] + 3., p[1], p[2]])));
        assert!(!touch(a, a.map(|p| [p[0], p[1], p[2] + 1.])));
        assert!(!touch(a, [[0.; 3]; 3]));
        assert!(!touch(
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            [[0.75, 0.75, 0.], [1.75, 0.75, 0.], [0.75, 1.75, 0.]]
        ));
        assert_eq!(point_triangle_distance([0.5, 0.5, 2.], a), 2.);
    }
}
