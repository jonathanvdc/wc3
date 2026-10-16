// Adapted from Bevy v0.19.1 pbr.wgsl (MIT OR Apache-2.0).
#import bevy_pbr::{
    pbr_types,
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
    decal::clustered::apply_decals,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

#ifdef VISIBILITY_RANGE_DITHER
#import bevy_pbr::pbr_functions::visibility_range_dither;
#endif

#ifdef MESHLET_MESH_MATERIAL_PASS
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output
#endif

#ifdef OIT_ENABLED
#import bevy_core_pipeline::oit::oit_draw
#endif // OIT_ENABLED

#ifdef FORWARD_DECAL
#import bevy_pbr::decal::forward::get_forward_decal_info
#endif

struct UniversalCombiner { uv_transform: mat3x3<f32>, mode: vec4<u32>, clip: vec4<f32>, };
@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> combiner: UniversalCombiner;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var secondary: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var secondary_sampler: sampler;
struct UniversalWorld { flags: vec4<u32>, };
@group(#{MATERIAL_BIND_GROUP}) @binding(110) var<uniform> world: UniversalWorld;
#import bevy_pbr::pbr_bindings
#import "embedded://bevy_wc3/shaders/universal_hd.wgsl"::apply_hd

@fragment
fn fragment(
#ifdef MESHLET_MESH_MATERIAL_PASS
    @builtin(position) frag_coord: vec4<f32>,
#else
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
#endif
) -> FragmentOutput {
#ifdef MESHLET_MESH_MATERIAL_PASS
    let vertex_output = resolve_vertex_output(frag_coord);
    let is_front = true;
#endif

    var in = vertex_output;

    // If we're in the crossfade section of a visibility range, conditionally
    // discard the fragment according to the visibility pattern.
#ifdef VISIBILITY_RANGE_DITHER
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif

#ifdef FORWARD_DECAL
    let forward_decal_info = get_forward_decal_info(in);
    in.world_position = forward_decal_info.world_position;
    in.uv = forward_decal_info.uv;
#endif

    // World colors encode irradiance and a lighting transition, not albedo or
    // opacity. Keep the authored source intact in the mesh for other consumers.
    var baked_diffuse = vec3(0.0);
#ifdef VERTEX_COLORS
    if world.flags.x != 0u {
        baked_diffuse = in.color.rgb;
        in.color = vec4(1.0);
    }
#endif

    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    if world.flags.x != 0u {
        if world.flags.y != 0u {
            // Shader 4 ignores sampled map alpha. Explicit material opacity and
            // instance geometry tint remain authored StandardMaterial inputs.
            pbr_input.material.base_color.a = pbr_bindings::material.base_color.a;
        }
        pbr_input.material.emissive = vec4(pbr_input.material.emissive.rgb
            + pbr_input.material.base_color.rgb * baked_diffuse,
            pbr_input.material.emissive.a);
    }

    pbr_input = apply_hd(in, is_front, pbr_input);

    // Authored M2 legacy texture composition precedes Bevy illumination.
    if combiner.mode.x != 0u {
        var uv = vec2(0.0);
#ifdef VERTEX_UVS_B
        uv = (combiner.uv_transform * vec3(in.uv_b, 1.0)).xy;
#endif
        let second = textureSample(secondary, secondary_sampler, uv);
        var opacity = pbr_bindings::material.base_color.a;
#ifdef VERTEX_COLORS
        opacity *= in.color.a;
#endif
        var alpha = opacity;
        if combiner.mode.y != 0u { alpha = pbr_input.material.base_color.a; }
        if combiner.mode.z != 0u { alpha *= second.a; }
        let gain = 1.0 + f32(combiner.mode.w);
        if combiner.mode.z != 0u { alpha *= gain; }
        pbr_input.material.base_color = vec4(pbr_input.material.base_color.rgb * second.rgb * gain, alpha);
    }

    if pbr_input.material.base_color.a < combiner.clip.x { discard; }

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // clustered decals
    apply_decals(&pbr_input);

#ifdef PREPASS_PIPELINE
    // write the gbuffer, lighting pass id, and optionally normal and motion_vector textures
    let out = deferred_output(in, pbr_input);
#else
    // in forward mode, we calculate the lit color immediately, and then apply some post-lighting effects here.
    // in deferred mode the lit color and these effects will be calculated in the deferred lighting shader
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

#ifdef OIT_ENABLED
    let alpha_mode = pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if alpha_mode != pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE {
        // The fragments will only be drawn during the oit resolve pass.
        oit_draw(in.position, out.color);
        discard;
    }
#endif // OIT_ENABLED

#ifdef FORWARD_DECAL
        out.color.a = min(forward_decal_info.alpha, out.color.a);
#endif

        return out;
}
