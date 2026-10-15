// CPU-posed diagnostic meshes only: no Bevy skinning, morphs or tangents.
// RGB is not native skin color. R carries the retained homogeneous bone W;
// alpha remains the native top/bottom texture control. Part previews use R=1.
#import bevy_pbr::{
    mesh_functions,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

@vertex
fn vertex(input: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = mesh_functions::get_world_from_local(input.instance_index);
    out.world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local, vec4<f32>(input.position, 1.0));
    out.world_normal = mesh_functions::mesh_normal_local_to_world(input.normal, input.instance_index);
    // input.position is xyz/W, framed by an affine display transform. Multiplying
    // the clip result by W restores P * View * Display * native_homogeneous_point,
    // including the original perspective-correct UV/control/normal interpolation.
    out.position = position_world_to_clip(out.world_position.xyz) * input.color.r;
    out.uv = input.uv;
    out.color = input.color;
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = input.instance_index;
#endif
    return out;
}
