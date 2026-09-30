#import bevy_pbr::{
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}

@group(3) @binding(0) var particle_texture: texture_2d<f32>;
@group(3) @binding(1) var particle_sampler: sampler;

struct VertexInput {
    @location(0) corner: vec3<f32>,
    @location(3) center_size: vec4<f32>,
    @location(4) velocity_tail: vec4<f32>,
    @location(5) color: vec4<f32>,
    @location(6) uv_rect: vec4<f32>,
    @location(7) flags: vec4<f32>,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};
fn safe_normalize(value: vec3<f32>) -> vec3<f32> {
    let length = length(value);
    if length < 0.00001 { return vec3(0.0, 0.0, 0.0); }
    return value / length;
}
@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    let right = view.world_from_view[0].xyz;
    let up = view.world_from_view[1].xyz;
    let forward = -view.world_from_view[2].xyz;
    let velocity = input.velocity_tail.xyz;
    let half_size = input.center_size.w * 0.5;
    var center = input.center_size.xyz;
    var side = right * half_size;
    var vertical = up * half_size;
    if input.flags.x > 0.5 {
        let axis = safe_normalize(velocity);
        side = safe_normalize(cross(axis, forward)) * half_size;
        vertical = velocity * input.velocity_tail.w * 0.5;
        center -= vertical;
    } else if input.flags.y > 0.5 {
        let axis = safe_normalize(velocity);
        side = vec3(-axis.y, axis.x, 0.0) * half_size;
        vertical = axis * half_size;
    }
    let world = center + side * input.corner.x + vertical * input.corner.y;
    var output: VertexOutput;
    output.position = position_world_to_clip(world);
    output.uv = input.uv_rect.xy + vec2((input.corner.x + 1.0) * 0.5, (1.0 - input.corner.y) * 0.5) * input.uv_rect.zw;
    output.color = input.color;
    return output;
}
@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(particle_texture, particle_sampler, input.uv) * input.color;
#ifdef ALPHA_KEY
    if color.a < 0.5 { discard; }
#endif
    return color;
}
