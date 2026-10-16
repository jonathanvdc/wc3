#import "embedded://bevy_wc3/shaders/model_io.wgsl"::{VertexOutput,to_bevy,from_bevy}
#import "embedded://bevy_wc3/shaders/particle_composition.wgsl"::{particle,compose_particle}
// Adapted from Bevy v0.19.1 pbr.wgsl (MIT OR Apache-2.0).
#import bevy_pbr::{
    pbr_types,
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
    decal::clustered::apply_decals,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::FragmentOutput,
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::FragmentOutput,
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

#import "embedded://bevy_wc3/shaders/combiner.wgsl"::{combiner,secondary,secondary_sampler,tertiary,tertiary_sampler,compose}
struct World { flags: vec4<u32>, };
@group(#{MATERIAL_BIND_GROUP}) @binding(110) var<uniform> world: World;
#import bevy_pbr::pbr_bindings
#import "embedded://bevy_wc3/shaders/hd.wgsl"::apply_hd

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
    let vertex_output = from_bevy(resolve_vertex_output(frag_coord));
    let is_front = true;
#endif

    var in = vertex_output;

    // If we're in the crossfade section of a visibility range, conditionally
    // discard the fragment according to the visibility pattern.
#ifdef VISIBILITY_RANGE_DITHER
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif

#ifdef FORWARD_DECAL
    let forward_decal_info = get_forward_decal_info(to_bevy(in));
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
    var pbr_input = pbr_input_from_standard_material(to_bevy(in), is_front);

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

    // Compose native texture masks before Bevy evaluates illumination.
    if combiner.mode.x != 0u {
        var uv0=vec2(0.0);
        var uv1=vec2(0.0);
        var uv2=vec2(0.0);
#ifdef VERTEX_UVS_A
        uv0=(pbr_bindings::material.uv_transform*vec3(in.uv,1.0)).xy;
#endif
#ifdef VERTEX_UVS_B
        uv1=(combiner.uv_transform*vec3(in.uv_b,1.0)).xy;
#endif
    // Reuse raw coordinate streams while retaining an independent transform.
#ifdef VERTEX_UVS_A
    uv2=in.uv;
#endif
#ifdef VERTEX_UVS_B
    if combiner.coordinates.w==2u {uv2=in.uv_b;}
#endif
#ifdef VERTEX_UVS_C
    uv2=in.uv_c;
#endif
    uv2=(combiner.third_transform*vec3(uv2,1.0)).xy;
        var a=vec4(1.0);
        if (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT)!=0u {
            a=textureSample(pbr_bindings::base_color_texture,pbr_bindings::base_color_sampler,uv0);
        }
        var b=vec4(1.0);
        if combiner.coordinates.z>1u {b=textureSample(secondary,secondary_sampler,uv1);}
        var c=vec4(1.0);
        if combiner.coordinates.z>2u {c=textureSample(tertiary,tertiary_sampler,uv2);}
        var tint=pbr_bindings::material.base_color;
#ifdef VERTEX_COLORS
        tint*=in.color;
#endif
        let combined=compose(a,b,c,tint);
        pbr_input.material.base_color=combined.color;
        if (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT)!=0u {
            pbr_input.material.base_color=vec4(combined.color.rgb+combined.emission,combined.color.a);
        } else {
            pbr_input.material.emissive=vec4(pbr_input.material.emissive.rgb+combined.emission,pbr_input.material.emissive.a);
        }
    }

    // Primary texture/tint have already been sampled by Bevy. The extra
    // layers use independent per-particle streams and preserve Bevy lighting.
    if particle.mode.x!=0u {
        var b=vec4(1.0); var c=vec4(1.0);
#ifdef VERTEX_UVS_B
        b=textureSample(secondary,secondary_sampler,in.uv_b);
#endif
#ifdef VERTEX_UVS_C
        c=textureSample(tertiary,tertiary_sampler,in.uv_c);
#endif
        pbr_input.material.base_color=compose_particle(pbr_input.material.base_color,b,c,vec4(1.0));
    }

#ifdef MODEL_EDGE_FADE
    pbr_input.material.base_color.a*=in.edge_opacity;
#endif

    if pbr_input.material.base_color.a < combiner.clip.x { discard; }

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // clustered decals
    apply_decals(&pbr_input);

#ifdef PREPASS_PIPELINE
    // write the gbuffer, lighting pass id, and optionally normal and motion_vector textures
    let out = deferred_output(to_bevy(in), pbr_input);
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
