//! Mounted source/control-point diagnostics; no guessed attachment corrections.
use openshores_client::{
    asset_mount::AssetMount,
    model::ModelVariant,
    world::{
        native_body::{NativeDpAvatarGeometry, NativeDpPartJoints},
        native_dna::NativeDhDna,
    },
};

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

// Diagnostic upper bound on separation, NOT a complete triangle/triangle test.
fn vertex_triangle_distance(p: [f64; 3], [a, b, c]: [[f64; 3]; 3]) -> f64 {
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
            let delta: [f64; 3] = std::array::from_fn(|i| p[i] - (a[i] + d[i] * t));
            dot(delta, delta).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

fn segment_distance(p: [f64; 3], q: [f64; 3], r: [f64; 3], u: [f64; 3]) -> f64 {
    let d = sub(q, p);
    let e = sub(u, r);
    let offset = sub(p, r);
    let a = dot(d, d);
    let b = dot(d, e);
    let c = dot(d, offset);
    let f = dot(e, offset);
    let g = dot(e, e);
    let (s, t) = if a == 0. && g == 0. {
        (0., 0.)
    } else if a == 0. {
        (0., (f / g).clamp(0., 1.))
    } else if g == 0. {
        ((-c / a).clamp(0., 1.), 0.)
    } else {
        let denominator = a * g - b * b;
        let s = if denominator > 0. {
            ((b * f - c * g) / denominator).clamp(0., 1.)
        } else {
            0.
        };
        let t = (b * s + f) / g;
        if t < 0. {
            ((-c / a).clamp(0., 1.), 0.)
        } else if t > 1. {
            (((b - c) / a).clamp(0., 1.), 1.)
        } else {
            (s, t)
        }
    };
    let delta = std::array::from_fn(|i| offset[i] + s * d[i] - t * e[i]);
    dot(delta, delta).sqrt()
}

fn crosses_triangle(p: [f64; 3], q: [f64; 3], [a, b, c]: [[f64; 3]; 3]) -> bool {
    let n = cross(sub(b, a), sub(c, a));
    let direction = sub(q, p);
    let denominator = dot(n, direction);
    if denominator == 0. {
        return false;
    }
    let t = dot(n, sub(a, p)) / denominator;
    if !(0.0..=1.).contains(&t) {
        return false;
    }
    let point = std::array::from_fn(|i| p[i] + t * direction[i]);
    [(a, b), (b, c), (c, a)]
        .into_iter()
        .all(|(a, b)| dot(cross(sub(b, a), sub(point, a)), n) >= 0.)
}

fn triangle_distance(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> f64 {
    let edges = |t: [[f64; 3]; 3]| [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])];
    if edges(a).into_iter().any(|(p, q)| crosses_triangle(p, q, b))
        || edges(b).into_iter().any(|(p, q)| crosses_triangle(p, q, a))
    {
        return 0.;
    }
    let mut nearest = f64::INFINITY;
    for p in a {
        nearest = nearest.min(vertex_triangle_distance(p, b));
    }
    for p in b {
        nearest = nearest.min(vertex_triangle_distance(p, a));
    }
    for (p, q) in edges(a) {
        for (r, s) in edges(b) {
            nearest = nearest.min(segment_distance(p, q, r, s));
        }
    }
    nearest
}

fn mesh_distance(a: &[[[f64; 3]; 3]], b: &[[[f64; 3]; 3]]) -> f64 {
    let mut nearest = f64::INFINITY;
    for a in a {
        for b in b {
            let mut separation = 0.;
            for axis in 0..3 {
                let amin = a.iter().map(|p| p[axis]).fold(f64::INFINITY, f64::min);
                let amax = a.iter().map(|p| p[axis]).fold(f64::NEG_INFINITY, f64::max);
                let bmin = b.iter().map(|p| p[axis]).fold(f64::INFINITY, f64::min);
                let bmax = b.iter().map(|p| p[axis]).fold(f64::NEG_INFINITY, f64::max);
                separation += (amin - bmax).max(bmin - amax).max(0.).powi(2);
            }
            if separation > nearest * nearest {
                continue;
            }
            nearest = nearest.min(triangle_distance(*a, *b));
            if nearest == 0. {
                return 0.;
            }
        }
    }
    nearest
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; source/control-point diagnostic, no GPU"]
fn mounted_attachment_coordinate_diagnostics() {
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    for second in [false, true] {
        diagnose(&mount, second);
    }
}

fn diagnose(mount: &AssetMount, second: bool) {
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[12] = 4;
    dna.0[14] = 0x51;
    dna.0[20] = 0x60;
    if second {
        dna.0[1] = 2;
        dna.0[12] = 8;
        dna.0[14] = 0x91;
        dna.0[20] = 0x80;
        dna.0[5] = 2;
        dna.0[13] = 0x60;
        dna.0[15] = 0x40;
    }
    println!(
        "attachment fixture {}",
        if second { "many" } else { "base" }
    );
    let avatar = NativeDpAvatarGeometry::generate(mount, &dna, 0, 0).unwrap();
    if let Some(path) = std::env::var_os("OPENSHORES_PATCHED_BODY_RESOURCES") {
        use sha1::{Digest, Sha1};
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(
            format!("{:x}", Sha1::digest(&bytes)),
            "88a3051bd4490102bde7ebd8ebf59a53fa8421e2",
            "optional static comparison selects the documented patched body-resource artifact"
        );
        // The selected symbol-bearing patched reference's qInitResources0
        // passes ECX=3; its import trampoline is outside the mount profile.
        let resources = openshores_client::qt_resources::QtResources::open(&bytes, 3).unwrap();
        for generated in &avatar.surfaces {
            for source in &generated.sources {
                let wanted = source
                    .resource
                    .trim_start_matches(':')
                    .trim_start_matches('/');
                let mut found = 0;
                for (index, _) in resources.entries().iter().enumerate().filter(|(_, entry)| {
                    entry.path.trim_start_matches(':').trim_start_matches('/') == wanted
                }) {
                    let original = mount.read_resource(&source.resource).unwrap();
                    let patched = resources.read_entry(index).unwrap();
                    assert!(
                        original == patched,
                        "resource mismatch: {}",
                        source.resource
                    );
                    found += 1;
                    println!(
                        "source identity {}: equal {} ({} / {} bytes)",
                        source.resource,
                        original == patched,
                        original.len(),
                        patched.len()
                    );
                }
                assert!(
                    found > 0,
                    "comparison resource missing: {}",
                    source.resource
                );
            }
        }
    }
    let posed = avatar.posed_surfaces(60.).unwrap();
    let repairs = avatar.seam_repairs().unwrap();
    assert_eq!(repairs.len(), if second { 8 } else { 4 });
    for repair in &repairs {
        let first = &avatar.surfaces[repair.source_surfaces[0]].surface;
        let second = &avatar.surfaces[repair.source_surfaces[1]].surface;
        let g = &repair.surface.geometry;
        let offset = repair.first_vertex_count;
        assert_eq!(&g.positions[..offset], first.geometry.positions);
        assert_eq!(&g.positions[offset..], second.geometry.positions);
        assert_eq!(&g.weights[..offset], first.geometry.weights);
        assert_eq!(&g.weights[offset..], second.geometry.weights);
        assert_eq!(
            &g.uvs.as_ref().unwrap()[..offset],
            first.geometry.uvs.as_ref().unwrap()
        );
        assert_eq!(
            &g.uvs.as_ref().unwrap()[offset..],
            second.geometry.uvs.as_ref().unwrap()
        );
        assert_eq!(
            g.faces.len(),
            first.geometry.faces.len() + second.geometry.faces.len()
                - repair.removed_faces.iter().sum::<usize>()
                + repair.bridge_faces
        );
        let mut canonical = std::collections::BTreeMap::new();
        let ids: Vec<_> = g
            .positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let key = p.map(|v| {
                    let v = v as f32;
                    if v == 0. { 0 } else { v.to_bits() }
                });
                *canonical.entry(key).or_insert(i)
            })
            .collect();
        let mut edges = std::collections::BTreeMap::new();
        for face in &g.faces {
            let v = face.vertices.map(|i| ids[usize::from(i)]);
            let t = face.vertices.map(|i| g.positions[usize::from(i)]);
            let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
            assert!(dot(n, n) > 0., "repair introduced a degenerate triangle");
            for (a, b) in [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
                let entry = edges.entry((a.min(b), a.max(b))).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(
            edges.values().all(|e| *e == (2, 0)),
            "repair {:?} must be closed and consistently wound: {:?}",
            repair.source_surfaces,
            edges
                .iter()
                .filter(|(_, e)| **e != (2, 0))
                .take(12)
                .collect::<Vec<_>>()
        );
    }
    let repaired = avatar.posed_surfaces_with_seam_repair(60.).unwrap();
    for repair in &repairs {
        let surface = repaired
            .iter()
            .find(|s| s.surface_index == repair.source_surfaces[0])
            .unwrap();
        let mut edges = std::collections::BTreeMap::new();
        for triangle in surface.corners.chunks_exact(3) {
            let keys: Vec<_> = triangle
                .iter()
                .map(|c| {
                    std::array::from_fn::<_, 3, _>(|i| {
                        let v = c.position[i] / c.position[3];
                        if v == 0. { 0 } else { v.to_bits() }
                    })
                })
                .collect();
            for (a, b) in [(keys[0], keys[1]), (keys[1], keys[2]), (keys[2], keys[0])] {
                *edges.entry((a.min(b), a.max(b))).or_insert(0usize) += 1;
            }
        }
        assert!(
            edges.values().all(|n| *n == 2),
            "skinned repaired surface must remain closed"
        );
    }
    assert_eq!(repaired.len(), posed.len() - repairs.len());
    assert!(
        repaired.iter().flat_map(|s| &s.corners).all(|c| c
            .position
            .iter()
            .chain(&c.normal)
            .all(|v| v.is_finite())
            && c.position[3] > 0.)
    );
    let point = |c: &openshores_client::world::native_body::NativeDpAvatarCorner| -> [f64; 3] {
        std::array::from_fn(|i| f64::from(c.position[i]) / f64::from(c.position[3]))
    };
    println!("dimensions {:?}", avatar.body.dimensions());
    for (part_index, part) in avatar.body.parts().iter().enumerate() {
        if !matches!(
            part.joints,
            NativeDpPartJoints::Arm { .. } | NativeDpPartJoints::Leg { .. }
        ) {
            continue;
        }
        println!(
            "part{part_index}: {:?} connect {:?} joints {:?}",
            part.allocation.kind, part.connect_30, part.joints
        );
        let joint = match part.joints {
            NativeDpPartJoints::Arm { field_70, .. } => field_70,
            NativeDpPartJoints::Leg { field_68, .. } => field_68,
            _ => unreachable!(),
        };
        let indices = part.limb_bones().unwrap();
        let proximal = avatar.body.bones().slots[indices[1]].project(joint);
        let distal = avatar.body.bones().slots[indices[2]].project(joint);
        println!("  shared distal joint proximal {proximal:?}, distal {distal:?}");
        for axis in 0..3 {
            assert!((proximal[axis] - distal[axis]).abs() < 1e-10);
        }
        let surfaces: Vec<_> = posed
            .iter()
            .filter(|s| s.part_index == part_index)
            .collect();
        if let [first, second] = surfaces.as_slice() {
            let triangles =
                |s: &openshores_client::world::native_body::NativeDpPosedAvatarSurface| {
                    s.corners
                        .chunks_exact(3)
                        .map(|c| [point(&c[0]), point(&c[1]), point(&c[2])])
                        .collect::<Vec<_>>()
                };
            println!(
                "  posed triangle distance {}",
                mesh_distance(&triangles(first), &triangles(second))
            );
            let models: Vec<_> = avatar
                .surfaces
                .iter()
                .filter(|s| s.part_index == part_index)
                .collect();
            for model in &models {
                let cap = openshores_client::world::native_body::native_avatar_end_cap(
                    &model.surface,
                    joint,
                );
                println!(
                    "  cap {}: {:?}",
                    model.sources[0].resource,
                    cap.as_ref().map(|c| (c.faces.len(), c.perimeter.len()))
                );
                let g = &model.surface.geometry;
                let mut canonical = std::collections::BTreeMap::new();
                let ids: Vec<usize> = g
                    .positions
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let key = p.map(|x| {
                            let x = x as f32;
                            if x == 0. { 0 } else { x.to_bits() }
                        });
                        *canonical.entry(key).or_insert(i)
                    })
                    .collect();
                let mut edges = std::collections::BTreeMap::new();
                for face in &g.faces {
                    let v = face.vertices.map(|i| ids[usize::from(i)]);
                    for (a, b) in [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
                        *edges.entry((a.min(b), a.max(b))).or_insert(0usize) += 1;
                    }
                }
                let boundary: Vec<_> = edges
                    .iter()
                    .filter(|(_, n)| **n == 1)
                    .map(|(e, _)| *e)
                    .collect();
                println!(
                    "  boundary {}: {} edges, {} nonmanifold",
                    model.sources[0].resource,
                    boundary.len(),
                    edges.values().filter(|n| **n > 2).count()
                );
            }
            let triangles = |s: &openshores_client::world::native_body::NativeDpAvatarSurface| {
                s.surface
                    .geometry
                    .faces
                    .iter()
                    .map(|f| {
                        f.vertices
                            .map(|i| s.surface.geometry.positions[usize::from(i)])
                    })
                    .collect::<Vec<_>>()
            };
            println!(
                "  model-space triangle distance {}",
                mesh_distance(&triangles(models[0]), &triangles(models[1]))
            );
            let mut nearest = f64::INFINITY;
            for (from, to) in [(first, second), (second, first)] {
                let points = from
                    .corners
                    .iter()
                    .map(|c| (c.source_vertex, point(c)))
                    .collect::<std::collections::BTreeMap<_, _>>();
                let triangles: Vec<_> = to
                    .corners
                    .chunks_exact(3)
                    .map(|c| [point(&c[0]), point(&c[1]), point(&c[2])])
                    .collect();
                for p in points.values() {
                    for triangle in &triangles {
                        nearest = nearest.min(vertex_triangle_distance(*p, *triangle));
                    }
                }
            }
            println!("  vertex-to-triangle upper bound {nearest}");
        }
        for generated in avatar
            .surfaces
            .iter()
            .filter(|s| s.part_index == part_index)
        {
            let native = &generated.native;
            let mesh = &generated.sources[0].mesh;
            let g = &generated.surface.geometry;
            let used: std::collections::BTreeSet<_> = g
                .faces
                .iter()
                .flat_map(|f| f.vertices)
                .map(usize::from)
                .collect();
            let range = |points: &[[f64; 3]]| -> [[f64; 2]; 3] {
                std::array::from_fn(|axis| {
                    [
                        used.iter()
                            .map(|i| points[*i][axis])
                            .fold(f64::INFINITY, f64::min),
                        used.iter()
                            .map(|i| points[*i][axis])
                            .fold(f64::NEG_INFINITY, f64::max),
                    ]
                })
            };
            let loaded = mesh.loaded_positions(ModelVariant::N, 1.).unwrap();
            println!(
                "{} matrix {:?} offset {:?} connection {:?} scale {} radial/height {:?} loaded {:?} deformed {:?}",
                generated.sources[0].resource,
                mesh.matrix,
                native.deformation.base.fields_1c0,
                native.deformation.base.fields_1f4,
                native.deformation.base.field_140,
                [
                    native.deformation.field_170,
                    native.deformation.field_178,
                    native.deformation.field_180,
                    native.deformation.field_188
                ],
                range(&loaded),
                range(&g.positions)
            );
            for group in &mesh.vertex_groups {
                let native_group =
                    openshores_client::model::NativeBodyVertexGroup::from_group(group).unwrap();
                println!(
                    "  group {} native kind {} pivot {:?} count {}",
                    String::from_utf8_lossy(group.name.as_deref().unwrap_or_default()),
                    native_group.kind_34,
                    native_group.point_28,
                    group.members.as_ref().map_or(0, Vec::len)
                );
            }
        }
    }
}

#[test]
fn vertex_triangle_upper_bound_handles_face_edge_and_degenerate_cases() {
    let t = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    assert_eq!(vertex_triangle_distance([0.25, 0.25, 0.], t), 0.);
    assert_eq!(vertex_triangle_distance([0.25, 0.25, 2.], t), 2.);
    assert_eq!(vertex_triangle_distance([2., 0., 0.], t), 1.);
    assert_eq!(vertex_triangle_distance([0., 0., 1.], [[0.; 3]; 3]), 1.);
}

#[test]
fn contact_diagnostic_handles_crossing_edges_piercing_faces_and_separation() {
    assert_eq!(
        segment_distance([-1., 0., 0.], [1., 0., 0.], [0., -1., 0.], [0., 1., 0.]),
        0.
    );
    assert_eq!(
        segment_distance([0.; 3], [0.; 3], [1., 0., 0.], [2., 0., 0.]),
        1.
    );
    let a = [[0., 0., 0.], [2., 0., 0.], [0., 2., 0.]];
    let b = [[0.5, 0.5, -1.], [0.5, 0.5, 1.], [3., 3., 1.]];
    assert_eq!(triangle_distance(a, b), 0.);
    assert_eq!(triangle_distance(a, a.map(|p| [p[0], p[1], p[2] + 2.])), 2.);
    let coplanar = [[0.5, -1., 0.], [0.5, 2., 0.], [1.5, 2., 0.]];
    assert!(triangle_distance(a, coplanar) < 1e-14);
    assert_eq!(triangle_distance([[0.; 3]; 3], [[1., 0., 0.]; 3]), 1.);
}
