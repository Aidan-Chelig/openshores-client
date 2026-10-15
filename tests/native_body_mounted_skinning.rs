//! Mounted shader contract only; never executes the supplied application.
use openshores_client::{asset_mount::AssetMount, har::HarArchive};

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; shader contract, no GPU"]
fn mounted_body_vertex_shader_matches_cpu_skinning_contract() {
    let mount =
        AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").unwrap()).unwrap();
    let bytes = mount.read_resource(":glsl.har").unwrap();
    let archive = HarArchive::open_sequential(&bytes).unwrap();
    let shader = archive.read_entry("Body.v.glsl").unwrap();
    let text = std::str::from_utf8(&shader).unwrap();
    let compact: String = text.split_whitespace().collect();
    for anchor in [
        "#defineMaxBones61",
        "for(intib=0;ib<4;ib++)",
        "if(boneWeight[ib]!=0)",
        "intindex=int(boneIndex[ib]);",
        "vec4bonePosition=bM[index]*vec4(position,1);",
        "mat3rotate=mat3(bM[index]);",
        "vec3boneNormal=rotate*normal;",
        "netPosition+=boneWeight[ib]*bonePosition;",
        "netNormal+=boneWeight[ib]*boneNormal;",
        "netNormal=normalize(netNormal);",
        "gl_Position=mv*netPosition;",
    ] {
        assert!(
            compact.contains(anchor),
            "missing shader contract: {anchor}"
        );
    }
    // This test establishes the supplied shader's operations, not runtime
    // equivalence, neutral-pose construction or complete native assembly.
}
