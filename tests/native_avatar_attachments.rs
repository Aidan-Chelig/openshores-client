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

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; source/control-point diagnostic, no GPU"]
fn mounted_attachment_coordinate_diagnostics() {
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    let mut dna = NativeDhDna([0; 24]);
    dna.0[1] = 1;
    dna.0[12] = 4;
    dna.0[14] = 0x51;
    dna.0[20] = 0x60;
    let avatar = NativeDpAvatarGeometry::generate(&mount, &dna, 0, 0).unwrap();
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
    let point = |c: &openshores_client::world::native_body::NativeDpAvatarCorner| -> [f64; 3] {
        std::array::from_fn(|i| f64::from(c.position[i]) / f64::from(c.position[3]))
    };
    println!("dimensions {:?}", avatar.body.dimensions());
    for (part_index, part) in avatar.body.parts().iter().enumerate() {
        if part_index < 3 {
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
