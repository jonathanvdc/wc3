// Adapted from Bevy v0.19.1: crates/bevy_pbr/src/render/mesh.wgsl
// https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/mesh.wgsl
// Upstream license: MIT OR Apache-2.0.
// WC3 changes: local Vertex input adds JOINTS_1/WEIGHTS_1 at locations 8/9;
// wc3_skin_model adds those four weighted matrices.
// The forward/prepass outputs and all other logic follow the pinned Bevy source.
// When upgrading Bevy, diff this file against the matching upstream version.

#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph::{morph_position, morph_normal, morph_tangent},
    forward_io::VertexOutput,
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
#ifdef WC3_EXTRA_INFLUENCES
    @location(8) extra_joint_indices: vec4<u32>,
    @location(9) extra_joint_weights: vec4<f32>,
#endif
#endif
#ifdef MORPH_TARGETS
    @builtin(vertex_index) index: u32,
#endif
};
// WC3 END: vertex input.

// WC3 BEGIN: eight-influence skinning functions.
#ifdef SKINNED
#ifdef WC3_EXTRA_INFLUENCES
fn wc3_skin_model(indices: vec4<u32>, weights: vec4<f32>, extra_indices: vec4<u32>, extra_weights: vec4<f32>, instance_index: u32) -> mat4x4<f32> {
    var result = skinning::skin_model(indices, weights, instance_index);
#ifdef SKINS_USE_UNIFORM_BUFFERS
    result += extra_weights.x * skinning::joint_matrices.data[extra_indices.x]
        + extra_weights.y * skinning::joint_matrices.data[extra_indices.y]
        + extra_weights.z * skinning::joint_matrices.data[extra_indices.z]
        + extra_weights.w * skinning::joint_matrices.data[extra_indices.w];
#else
    let skin_index = mesh[instance_index].current_skin_index;
    result += extra_weights.x * skinning::joint_matrices[skin_index + extra_indices.x]
        + extra_weights.y * skinning::joint_matrices[skin_index + extra_indices.y]
        + extra_weights.z * skinning::joint_matrices[skin_index + extra_indices.z]
        + extra_weights.w * skinning::joint_matrices[skin_index + extra_indices.w];
#endif
    return result;
}

#endif
#endif
// WC3 END: eight-influence skinning functions.

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
#ifdef WC3_EXTRA_INFLUENCES
    var world_from_local = wc3_skin_model(
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
    out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
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

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
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
