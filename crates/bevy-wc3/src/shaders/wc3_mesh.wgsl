#import "embedded://bevy_wc3/shaders/model_io.wgsl"::VertexOutput
#ifdef MODEL_GENERATED_COORDS
#import "embedded://bevy_wc3/shaders/material_access.wgsl"::load_combiner
#import bevy_pbr::mesh_view_bindings::view
#endif
// Adapted from Bevy v0.19.1: crates/bevy_pbr/src/render/mesh.wgsl
// https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/mesh.wgsl
// Upstream license: MIT OR Apache-2.0.
// WC3 changes: local Vertex input adds JOINTS_1/WEIGHTS_1 at locations 8/9;
// extra_skin_model adds those four weighted matrices.
// Generated sphere coordinates reuse the UV outputs and skinned normal stream.
// The remaining forward/prepass interface follows the pinned Bevy source.
// When upgrading Bevy, diff this file against the matching upstream version.

#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph::{morph_position, morph_normal, morph_tangent},
    view_transformations::position_world_to_clip,
}

// WC3 BEGIN: vertex input with a second joint set.
struct Vertex {
    @builtin(instance_index) instance_index: u32,
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(6) joint_indices: vec4<u32>,
    @location(7) joint_weights: vec4<f32>,
#ifdef EXTRA_SKIN_INFLUENCES
    @location(8) extra_joint_indices: vec4<u32>,
    @location(9) extra_joint_weights: vec4<f32>,
#endif
#endif
#ifdef MORPH_TARGETS
    @builtin(vertex_index) index: u32,
#endif
#ifdef VERTEX_UVS_C
    @location(10) uv_c: vec2<f32>,
#endif
#ifdef WORLD_LAYER_ALPHA
    @location(11) second_color: vec4<f32>,
#endif
#ifdef VERTEX_UVS_D
    @location(12) uv_d: vec2<f32>,
#endif
};
// WC3 END: vertex input.

#ifdef SKINNED
#import "embedded://bevy_wc3/shaders/extra_skinning.wgsl"::{skin_normal_finite, skin_tangent_finite}
#ifdef EXTRA_SKIN_INFLUENCES
#import "embedded://bevy_wc3/shaders/extra_skinning.wgsl"::{extra_skin_model, extra_skin_prev_model}
#endif
#endif

#ifdef MORPH_TARGETS
// The instance_index parameter must match vertex_in.instance_index. This is a work around for a wgpu dx12 bug.
// See https://github.com/gfx-rs/naga/issues/2416
fn morph_vertex(vertex_in: Vertex, instance_index: u32) -> Vertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;

    let weight_count = bevy_pbr::morph::layer_count(instance_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i, instance_index);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph_position(vertex_index, i, instance_index);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * morph_normal(vertex_index, i, instance_index);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph_tangent(vertex_index, i, instance_index), 0.0);
#endif
    }
    return vertex;
}
#endif

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph, vertex_no_morph.instance_index);
#else
    var vertex = vertex_no_morph;
#endif

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

#ifdef SKINNED
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
#ifdef EXTRA_SKIN_INFLUENCES
    var world_from_local = extra_skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex.extra_joint_indices,
        vertex.extra_joint_weights,
        vertex_no_morph.instance_index
    );
#else
    var world_from_local = skinning::skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex_no_morph.instance_index
    );
#endif
#else
    var world_from_local = mesh_world_from_local;
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skin_normal_finite(world_from_local, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif
#ifdef VERTEX_UVS_C
    out.uv_c = vertex.uv_c;
#endif
#ifdef VERTEX_UVS_D
    out.uv_d = vertex.uv_d;
#endif
#ifdef WORLD_LAYER_ALPHA
    out.world_layer_alpha = vertex.second_color.a;
#endif

#ifdef MODEL_GENERATED_COORDS
#ifdef VERTEX_NORMALS
#ifdef SKINNED
    let env_world_normal=skin_normal_finite(world_from_local,vertex.normal);
#else
    let env_world_normal=mesh_functions::mesh_normal_local_to_world(vertex.normal,vertex_no_morph.instance_index);
#endif
    let view_position=(view.view_from_world*out.world_position).xyz;
    let view_normal=normalize((view.view_from_world*vec4(env_world_normal,0.0)).xyz);
#ifdef MODEL_EDGE_FADE
    let facing=max(dot(-normalize(view_position),view_normal),0.0);
    out.edge_opacity=clamp(2.7*facing*facing-0.4,0.0,1.0);
#endif
    let combiner = load_combiner(vertex_no_morph.instance_index);
    if combiner.coordinates.x!=0u || combiner.coordinates.y!=0u || combiner.coordinates.w==1u {
    let reflected=reflect(-normalize(view_position),view_normal)+vec3(0.0,0.0,1.0);
    let generated=0.5*reflected.xy/max(length(reflected),0.000001)+vec2(0.5);
#ifdef VERTEX_UVS_A
    if combiner.coordinates.x!=0u {out.uv=generated;}
#endif
#ifdef VERTEX_UVS_B
    if combiner.coordinates.y!=0u {out.uv_b=generated;}
    if combiner.coordinates.y==2u {out.uv_b=reflect(normalize(view_position),view_normal).xy;}
#endif
#ifdef VERTEX_UVS_C
    if combiner.coordinates.w==1u {out.uv_c=generated;}
#endif
    }
#endif
#endif

#ifdef VERTEX_TANGENTS
#ifdef SKINNED
    out.world_tangent = skin_tangent_finite(world_from_local, vertex.tangent);
#else
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex_no_morph.instance_index, mesh_world_from_local[3]);
#endif

    return out;
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
#ifdef VERTEX_COLORS
    return mesh.color;
#else
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
#endif
}
