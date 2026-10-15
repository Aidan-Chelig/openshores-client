// Native two-region skin composition with diagnostic modern Bevy lighting.
// Clothes are disabled for the explicit owner4068=0 preview fixture.
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var atlas: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var atlas_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var<uniform> offsets: vec4<f32>;

@fragment
fn fragment(input: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let top = textureSample(atlas, atlas_sampler, input.uv + offsets.xy);
    let bottom = textureSample(atlas, atlas_sampler, input.uv + offsets.zw);
    let skin = mix(bottom, top, input.color.a);
    var vertex = input;
    // The alpha vertex attribute carries the native top-face control, NOT opacity.
    vertex.color = vec4<f32>(1.0);
    var pbr = pbr_input_from_standard_material(vertex, is_front);
    pbr.material.base_color = skin;
    pbr.material.base_color = alpha_discard(pbr.material, pbr.material.base_color);
    var out: FragmentOutput;
    if (pbr.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) != 0u {
        out.color = pbr.material.base_color;
    } else {
        out.color = apply_pbr_lighting(pbr);
    }
    out.color = main_pass_post_lighting_processing(pbr, out.color);
    return out;
}
