// Adapted from Bevy v0.19.1 forward_io.wgsl and prepass_io.wgsl.
// Upstream license: MIT OR Apache-2.0. Keep conditional members synchronized.
// Optional semantic varyings extend Bevy IO without changing its PBR inputs.
#ifdef PREPASS_PIPELINE
#import bevy_pbr::prepass_io
struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,

#ifdef VERTEX_UVS_A
    @location(0) uv: vec2<f32>,
#endif

#ifdef VERTEX_UVS_B
    @location(1) uv_b: vec2<f32>,
#endif

#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
    @location(2) world_normal: vec3<f32>,
#ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
#endif
#endif // NORMAL_PREPASS_OR_DEFERRED_PREPASS

    @location(4) world_position: vec4<f32>,
#ifdef MOTION_VECTOR_PREPASS
    @location(5) previous_world_position: vec4<f32>,
#endif

#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    @location(6) unclipped_depth: f32,
#endif // UNCLIPPED_DEPTH_ORTHO_EMULATION
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    @location(7) instance_index: u32,
#endif

#ifdef VERTEX_COLORS
    @location(8) color: vec4<f32>,
#endif

#ifdef VISIBILITY_RANGE_DITHER
    @location(9) @interpolate(flat) visibility_range_dither: i32,
#endif  // VISIBILITY_RANGE_DITHER
#ifdef VERTEX_UVS_C
    @location(10) uv_c: vec2<f32>,
#endif
#ifdef MODEL_EDGE_FADE
    @location(11) edge_opacity: f32,
#endif
}
#else
#import bevy_pbr::forward_io
struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) world_tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    @location(6) @interpolate(flat) instance_index: u32,
#endif
#ifdef VISIBILITY_RANGE_DITHER
    @location(7) @interpolate(flat) visibility_range_dither: i32,
#endif
#ifdef VERTEX_UVS_C
    @location(10) uv_c: vec2<f32>,
#endif
#ifdef MODEL_EDGE_FADE
    @location(11) edge_opacity: f32,
#endif
}
#endif
#ifdef PREPASS_PIPELINE
fn to_bevy(in: VertexOutput) -> prepass_io::VertexOutput {
    var out: prepass_io::VertexOutput;
    out.position=in.position;
    out.world_position=in.world_position;
#ifdef VERTEX_UVS_A
    out.uv=in.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b=in.uv_b;
#endif
#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
    out.world_normal=in.world_normal;
#ifdef VERTEX_TANGENTS
    out.world_tangent=in.world_tangent;
#endif
#endif
#ifdef MOTION_VECTOR_PREPASS
    out.previous_world_position=in.previous_world_position;
#endif
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.unclipped_depth=in.unclipped_depth;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index=in.instance_index;
#endif
#ifdef VERTEX_COLORS
    out.color=in.color;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither=in.visibility_range_dither;
#endif
    return out;
}
fn from_bevy(in: prepass_io::VertexOutput) -> VertexOutput {
    var out: VertexOutput;
    out.position=in.position;
    out.world_position=in.world_position;
#ifdef VERTEX_UVS_A
    out.uv=in.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b=in.uv_b;
#endif
#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
    out.world_normal=in.world_normal;
#ifdef VERTEX_TANGENTS
    out.world_tangent=in.world_tangent;
#endif
#endif
#ifdef MOTION_VECTOR_PREPASS
    out.previous_world_position=in.previous_world_position;
#endif
#ifdef UNCLIPPED_DEPTH_ORTHO_EMULATION
    out.unclipped_depth=in.unclipped_depth;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index=in.instance_index;
#endif
#ifdef VERTEX_COLORS
    out.color=in.color;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither=in.visibility_range_dither;
#endif
#ifdef VERTEX_UVS_C
    out.uv_c=vec2(0.0);
#endif
#ifdef MODEL_EDGE_FADE
    out.edge_opacity=1.0;
#endif
    return out;
}
#endif
#ifndef PREPASS_PIPELINE
fn to_bevy(in: VertexOutput) -> forward_io::VertexOutput {
    var out: forward_io::VertexOutput;
    out.position=in.position;
    out.world_position=in.world_position;
    out.world_normal=in.world_normal;
#ifdef VERTEX_UVS_A
    out.uv=in.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b=in.uv_b;
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent=in.world_tangent;
#endif
#ifdef VERTEX_COLORS
    out.color=in.color;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index=in.instance_index;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither=in.visibility_range_dither;
#endif
    return out;
}
fn from_bevy(in: forward_io::VertexOutput) -> VertexOutput {
    var out: VertexOutput;
    out.position=in.position;
    out.world_position=in.world_position;
    out.world_normal=in.world_normal;
#ifdef VERTEX_UVS_A
    out.uv=in.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b=in.uv_b;
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent=in.world_tangent;
#endif
#ifdef VERTEX_COLORS
    out.color=in.color;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index=in.instance_index;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither=in.visibility_range_dither;
#endif
#ifdef VERTEX_UVS_C
    out.uv_c=vec2(0.0);
#endif
#ifdef MODEL_EDGE_FADE
    out.edge_opacity=1.0;
#endif
    return out;
}
#endif
