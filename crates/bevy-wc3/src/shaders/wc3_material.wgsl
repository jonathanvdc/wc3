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

struct Wc3HdUniform {
    fresnel_color: vec4<f32>,
    fresnel: vec4<f32>,
    maps: vec4<u32>,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> hd: Wc3HdUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var orm_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var orm_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var team_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var team_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(105) var environment_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(106) var environment_sampler: sampler;
#import bevy_pbr::{pbr_bindings, pbr_functions, mesh_view_bindings::view}

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

    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // HD maps use a shared, animated UV transform. Normal RG reconstructs Z
    // even when the source texture has RGB(A) channels.
#ifdef VERTEX_UVS_A
    if hd.maps.x != 0u {
        let uv = (pbr_bindings::material.uv_transform * vec3(in.uv, 1.0)).xy;
#ifdef STANDARD_MATERIAL_NORMAL_MAP
#ifdef VERTEX_TANGENTS
        let sampled = textureSampleBias(pbr_bindings::normal_map_texture,
            pbr_bindings::normal_map_sampler, uv, view.mip_bias).rgb;
        let xy = sampled.rg * 2.0 - 1.0;
        let nt = vec3(xy, sqrt(max(0.0, 1.0 - dot(xy, xy)))) * 0.5 + vec3(0.5);
        let flags = pbr_input.material.flags & ~pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP;
        let double_sided = (flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;
        let tbn = pbr_functions::calculate_tbn_mikktspace(in.world_normal, in.world_tangent);
        pbr_input.N = pbr_functions::apply_normal_mapping(flags, tbn, double_sided, is_front, nt);
#endif
#endif
        var team = vec3(1.0);
        if hd.maps.z != 0u {
            team = textureSampleBias(team_map, team_sampler, uv, view.mip_bias).rgb;
        }
        if hd.maps.y != 0u && hd.maps.z != 0u {
            let mask = textureSampleBias(orm_map, orm_sampler, uv, view.mip_bias).a;
            // Team color continuously modulates diffuse RGB through the ORM
            // alpha mask while preserving diffuse alpha.
            pbr_input.material.base_color = vec4(pbr_input.material.base_color.rgb
                * mix(vec3(1.0), team, mask), pbr_input.material.base_color.a);
        }
        let rim = pow(1.0 - clamp(dot(pbr_input.N, pbr_input.V), 0.0, 1.0), 5.0);
        let fresnel_color = mix(hd.fresnel_color.rgb, team, clamp(hd.fresnel.y, 0.0, 1.0));
        var reflection = vec3(0.0);
        if hd.maps.w != 0u {
            let direction = normalize(reflect(-pbr_input.V, pbr_input.N));
            let env_uv = vec2(atan2(direction.x, direction.y), -asin(clamp(direction.z, -1.0, 1.0)))
                * vec2(0.15915494, 0.31830989) + vec2(0.5);
            let environment = textureSampleBias(environment_map, environment_sampler, env_uv,
                view.mip_bias + pbr_input.material.perceptual_roughness * 5.0).rgb;
            let f0 = mix(vec3(0.04), pbr_input.material.base_color.rgb, pbr_input.material.metallic);
            reflection = environment * mix(f0, vec3(1.0), rim)
                * pow(1.0 - pbr_input.material.perceptual_roughness, 2.0);
        }
        // The stored controls drive an explicit approximation, independent of
        // Bevy's physical reflectance. Exact game Fresnel/IBL remains unverified.
        pbr_input.material.emissive = vec4(pbr_input.material.emissive.rgb
            + (fresnel_color * rim * hd.fresnel.x + reflection)
                * pbr_bindings::material.base_color.rgb, pbr_input.material.emissive.a);
    }
#endif

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
