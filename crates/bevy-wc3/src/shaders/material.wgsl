#import "embedded://bevy_wc3/shaders/model_io.wgsl"::{VertexOutput,to_bevy,from_bevy}
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

#import "embedded://bevy_wc3/shaders/combiner.wgsl"::{combiner,secondary,secondary_sampler,tertiary,tertiary_sampler,fourth,fourth_sampler,compose}
#import "embedded://bevy_wc3/shaders/world_composition.wgsl"::{world,world_color}
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
    var world_vertex_alpha = 1.0;
#ifdef VERTEX_COLORS
    if world.flags.x != 0u {
        baked_diffuse = in.color.rgb;
        world_vertex_alpha = in.color.a;
        in.color = vec4(1.0);
    }
#endif

    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input=pbr_input_from_standard_material(to_bevy(in),is_front);

    if world.flags.x != 0u && world.flags.y != 0u {
        pbr_input.material.base_color.a = pbr_bindings::material.base_color.a;
    }

    if world.composition.x != 0u {
        var layer_alpha=1.0;
#ifdef WORLD_LAYER_ALPHA
        layer_alpha=in.world_layer_alpha;
#endif
        let composed = world_color(in, layer_alpha);
        pbr_input.material.base_color = composed.color;
        if (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT)!=0u {
            pbr_input.material.base_color=vec4(composed.color.rgb+composed.emission,composed.color.a);
        } else {
            pbr_input.material.emissive = vec4(pbr_input.material.emissive.rgb+composed.emission,pbr_input.material.emissive.a);
        }
    }

    if world.flags.x != 0u {
        var transition=world_vertex_alpha;
        if world.flags.z==1u {transition=0.0;}
        if world.flags.z==2u {transition=1.0;}
        let illumination=2.0*baked_diffuse+world.ambient.rgb*(1.0-transition);
        pbr_input.material.emissive = vec4(pbr_input.material.emissive.rgb
            + pbr_input.material.base_color.rgb * illumination,
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
    uv1=(combiner.uv_transform*vec3(in.uv,1.0)).xy;
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
    if combiner.coordinates.w==1u || combiner.coordinates.w==3u {uv2=in.uv_c;}
#endif
    uv2=(combiner.third_transform*vec3(uv2,1.0)).xy;
    var d=vec4(1.0);
    if combiner.coordinates.z>3u {
    var uv3=vec2(0.0);
#ifdef VERTEX_UVS_A
    uv3=in.uv;
#endif
#ifdef VERTEX_UVS_B
    if combiner.additional_coordinates.x==2u {uv3=in.uv_b;}
#endif
#ifdef VERTEX_UVS_C
    if combiner.additional_coordinates.x==3u {uv3=in.uv_c;}
#endif
    uv3=(combiner.fourth_transform*vec3(uv3,1.0)).xy;
    d=textureSample(fourth,fourth_sampler,uv3);
    }

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
        let combined=compose(a,b,c,d,tint);
        pbr_input.material.base_color=combined.color;
        if (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT)!=0u {
            pbr_input.material.base_color=vec4(combined.color.rgb+combined.emission,combined.color.a);
        } else {
            pbr_input.material.emissive=vec4(pbr_input.material.emissive.rgb+combined.emission,pbr_input.material.emissive.a);
        }
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
