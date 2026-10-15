//! Actual user-supplied body-group records, entirely offline and read-only.
use openshores_client::{
    asset_mount::AssetMount,
    model::NativeBodyVertexGroup,
    world::{native_body::NativeDpHandResource, native_dna::NativeDhDna},
};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION pointing to a user-provided copy"]
fn mounted_body_groups_preserve_native_membership_and_pivot_separation() {
    let mount = AssetMount::open(
        std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation path"),
    )
    .unwrap();
    let mut groups = 0;
    let mut keys = 0;
    let mut pivots = 0;
    for selector in 0..=64 {
        let mut dna = NativeDhDna([0; 24]);
        dna.0[16] = selector / 8;
        dna.0[11] = (selector % 8) << 5;
        let resource =
            NativeDpHandResource::new(&dna, if selector == 64 { 0x41 } else { 0 }, false);
        let model = resource.read_model(&mount).unwrap();
        for mesh in model.meshes {
            for raw in mesh.vertex_groups {
                let parsed = NativeBodyVertexGroup::from_group(&raw).unwrap();
                groups += 1;
                pivots += usize::from(parsed.point_28.iter().any(|v| *v != 0.));
                let members = raw.members.as_deref().unwrap_or(&[]);
                // Native single-record groups never insert a QMap weight.
                if members.len() <= 1 {
                    assert_eq!(parsed.entries().count(), 0);
                }
                for (point, weight) in parsed.entries() {
                    keys += 1;
                    assert!(members.iter().any(|member| member.position == point));
                    assert_eq!(
                        parsed.matching_weight(point.map(f64::from)).unwrap(),
                        Some(weight)
                    );
                    assert!(weight.is_finite());
                }
                assert!(parsed.entries().count() <= members.len());
            }
        }
    }
    assert!(groups > 0 && keys > 0 && pivots > 0);
    eprintln!(
        "Mounted body groups: {groups} parsed, {keys} native position keys, {pivots} nonzero pivots. No weighted-deformation or rendered-avatar parity claim."
    );
}
