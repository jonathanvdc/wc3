// Adapted from Bevy v0.19.1 pbr_prepass.wgsl (MIT OR Apache-2.0).
#import bevy_pbr::{
    pbr_prepass_functions,
    pbr_bindings,
    pbr_bindings::material,
    pbr_types,
    pbr_functions,
    pbr_functions::SampleBias,
    prepass_io,
    mesh_bindings::mesh,
    mesh_view_bindings::view,
}

#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d}

#ifdef MESHLET_MESH_MATERIAL_PASS
#import bevy_pbr::meshlet_visibility_buffer_resolve::resolve_vertex_output
#endif

#ifdef BINDLESS
#import bevy_pbr::pbr_bindings::material_indices
#endif  // BINDLESS

#import "embedded://bevy_wc3/shaders/combiner.wgsl"::{combiner,secondary,secondary_sampler,tertiary,tertiary_sampler,compose}
struct World { flags: vec4<u32>, };
@group(#{MATERIAL_BIND_GROUP}) @binding(110) var<uniform> world: World;

struct HdUniform { fresnel_color: vec4<f32>, fresnel: vec4<f32>, maps: vec4<u32>, };
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var<uniform> hd: HdUniform;

// Share composition with the color pass, including masks that are not opacity.
fn composed_alpha(in:prepass_io::VertexOutput)->f32 {
    var uv0=vec2(0.0); var uv1=vec2(0.0); var uv2=vec2(0.0);
#ifdef VERTEX_UVS_A
    uv0=(pbr_bindings::material.uv_transform*vec3(in.uv,1.0)).xy;
    uv2=(combiner.third_transform*vec3(in.uv,1.0)).xy;
#endif
#ifdef VERTEX_UVS_B
    uv1=(combiner.uv_transform*vec3(in.uv_b,1.0)).xy;
#endif
    var a=vec4(1.0);
    if (pbr_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT)!=0u {
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
    return compose(a,b,c,tint).color.a;
}

#ifdef PREPASS_FRAGMENT
@fragment
fn fragment(
#ifdef MESHLET_MESH_MATERIAL_PASS
    @builtin(position) frag_coord: vec4<f32>,
#else
    vertex_output: prepass_io::VertexOutput,
    @builtin(front_facing) is_front: bool,
#endif
) -> prepass_io::FragmentOutput {
#ifdef MESHLET_MESH_MATERIAL_PASS
    var in = resolve_vertex_output(frag_coord);
    let is_front = true;
#else   // MESHLET_MESH_MATERIAL_PASS
    var in = vertex_output;
#ifdef VERTEX_COLORS
    if world.flags.x != 0u { in.color = vec4(1.0); }
#endif

#ifdef BINDLESS
    let slot = mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let flags = pbr_bindings::material_array[material_indices[slot].material].flags;
    let uv_transform = pbr_bindings::material_array[material_indices[slot].material].uv_transform;
#else   // BINDLESS
    var flags = pbr_bindings::material.flags;

    let uv_transform = pbr_bindings::material.uv_transform;
#endif  // BINDLESS

    // If we're in the crossfade section of a visibility range, conditionally
    // discard the fragment according to the visibility pattern.
#ifdef VISIBILITY_RANGE_DITHER
    pbr_functions::visibility_range_dither(in.position, in.visibility_range_dither);
#endif  // VISIBILITY_RANGE_DITHER

    if world.flags.x != 0u && world.flags.y != 0u {
        let alpha_mode = flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
        if alpha_mode == pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK
            && pbr_bindings::material.base_color.a < pbr_bindings::material.alpha_cutoff { discard; }
    } else if combiner.mode.x == 0u {
        pbr_prepass_functions::prepass_alpha_discard(in);
    } else {
        let alpha_mode = flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
        if alpha_mode == pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK {
            let alpha = composed_alpha(in);
            if alpha < pbr_bindings::material.alpha_cutoff { discard; }
        }
    }
#endif  // MESHLET_MESH_MATERIAL_PASS

    var out: prepass_io::FragmentOutput;

#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.frag_depth = in.unclipped_depth;
#endif // UNCLIPPED_DEPTH_ORTHO_EMULATION

#ifdef NORMAL_PREPASS
    // NOTE: Unlit bit not set means == 0 is true, so the true case is if lit
    if (flags & pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        let double_sided = (flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

        let world_normal = pbr_functions::prepare_world_normal(
            in.world_normal,
            double_sided,
            is_front,
        );

        var normal = world_normal;

#ifdef VERTEX_UVS
#ifdef VERTEX_TANGENTS
#ifdef STANDARD_MATERIAL_NORMAL_MAP

// TODO: Transforming UVs mean we need to apply derivative chain rule for meshlet mesh material pass
#ifdef STANDARD_MATERIAL_NORMAL_MAP_UV_B
        let uv = (uv_transform * vec3(in.uv_b, 1.0)).xy;
#else
        let uv = (uv_transform * vec3(in.uv, 1.0)).xy;
#endif

        // Fill in the sample bias so we can sample from textures.
        var bias: SampleBias;
#ifdef MESHLET_MESH_MATERIAL_PASS
        bias.ddx_uv = in.ddx_uv;
        bias.ddy_uv = in.ddy_uv;
#else   // MESHLET_MESH_MATERIAL_PASS
        bias.mip_bias = view.mip_bias;
#endif  // MESHLET_MESH_MATERIAL_PASS

        var Nt =
#ifdef MESHLET_MESH_MATERIAL_PASS
            textureSampleGrad(
#else   // MESHLET_MESH_MATERIAL_PASS
            textureSampleBias(
#endif  // MESHLET_MESH_MATERIAL_PASS
#ifdef BINDLESS
                bindless_textures_2d[material_indices[slot].normal_map_texture],
                bindless_samplers_filtering[material_indices[slot].normal_map_sampler],
#else   // BINDLESS
                pbr_bindings::normal_map_texture,
                pbr_bindings::normal_map_sampler,
#endif  // BINDLESS
                uv,
#ifdef MESHLET_MESH_MATERIAL_PASS
                bias.ddx_uv,
                bias.ddy_uv,
#else   // MESHLET_MESH_MATERIAL_PASS
                bias.mip_bias,
#endif  // MESHLET_MESH_MATERIAL_PASS
            );
        if hd.maps.x != 0u {
            let packed=select(Nt.rg,Nt.ag,hd.maps.x==2u);
            let xy = packed * 2.0 - 1.0;
            Nt = vec4(vec3(xy, sqrt(max(0.0, 1.0 - dot(xy, xy)))) * 0.5 + vec3(0.5),Nt.a);
            flags &= ~pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP;
        }
        let TBN = pbr_functions::calculate_tbn_mikktspace(normal, in.world_tangent);

        normal = pbr_functions::apply_normal_mapping(
            flags,
            TBN,
            double_sided,
            is_front,
            Nt.rgb,
        );

#endif  // STANDARD_MATERIAL_NORMAL_MAP
#endif  // VERTEX_TANGENTS
#endif  // VERTEX_UVS

        out.normal = vec4(normal * 0.5 + vec3(0.5), 1.0);
    } else {
        out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);
    }
#endif // NORMAL_PREPASS

#ifdef MOTION_VECTOR_PREPASS
#ifdef MESHLET_MESH_MATERIAL_PASS
    out.motion_vector = in.motion_vector;
#else
    out.motion_vector = pbr_prepass_functions::calculate_motion_vector(in.world_position, in.previous_world_position);
#endif
#endif

    return out;
}
#else
@fragment
fn fragment(vertex_output: prepass_io::VertexOutput) {
    var in=vertex_output;
#ifdef VERTEX_COLORS
    if world.flags.x!=0u {in.color=vec4(1.0);}
#endif
    let flags = pbr_bindings::material.flags;
    let uv_transform = pbr_bindings::material.uv_transform;
    if world.flags.x != 0u && world.flags.y != 0u {
        let alpha_mode=flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
        if alpha_mode==pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK && pbr_bindings::material.base_color.a < pbr_bindings::material.alpha_cutoff {discard;}
    } else if combiner.mode.x == 0u {
        pbr_prepass_functions::prepass_alpha_discard(in);
    } else {
        let alpha_mode = flags & pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
        if alpha_mode == pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK {
            let alpha = composed_alpha(in);
            if alpha < pbr_bindings::material.alpha_cutoff { discard; }
        }
    }
}
#endif // PREPASS_FRAGMENT
