#import bevy_pbr::{mesh_view_bindings::view, view_transformations::position_world_to_clip}
#import "embedded://bevy_wc3/shaders/effect_lighting.wgsl"::shade_effect

struct Particle {
    position: vec4<f32>,
    velocity_tail: vec4<f32>,
    size_spin: vec4<f32>,
    color: vec4<f32>,
    offsets: vec4<f32>,
    scales: vec4<f32>,
    cells: vec4<u32>,
};
struct Emitter {
    axes: array<vec4<f32>, 2>,
    flags: vec4<u32>,
    atlas: vec4<u32>,
};
@group(2) @binding(0) var primary: texture_2d<f32>;
@group(2) @binding(1) var primary_sampler: sampler;
@group(2) @binding(2) var<uniform> emitter: Emitter;
@group(2) @binding(3) var<storage, read> particles: array<Particle>;
@group(2) @binding(4) var secondary: texture_2d<f32>;
@group(2) @binding(5) var secondary_sampler: sampler;
@group(2) @binding(6) var tertiary: texture_2d<f32>;
@group(2) @binding(7) var tertiary_sampler: sampler;
struct VertexInput {
    @location(0) corner: vec3<f32>,
    @location(3) particle_index: u32,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
    @location(4) uv_secondary: vec2<f32>,
    @location(5) uv_tertiary: vec2<f32>,
};
fn safe_normalize(value: vec3<f32>) -> vec3<f32> {
    let magnitude = length(value);
    if magnitude == 0.0 { return vec3(0.0); }
    return value / magnitude;
}
@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    let particle = particles[input.particle_index & 0x7fffffffu];
    let tail = (input.particle_index & 0x80000000u) != 0u;
    let camera_back = safe_normalize(view.world_from_view[2].xyz);
    var right = safe_normalize(view.world_from_view[0].xyz);
    var up = safe_normalize(view.world_from_view[1].xyz);
    var center = particle.position.xyz;
    let velocity = particle.velocity_tail.xyz;
    if emitter.flags.x != 0u {
        right = emitter.axes[0].xyz;
        up = emitter.axes[1].xyz;
    } else if emitter.flags.y != 0u {
        let projected = velocity - camera_back * dot(velocity, camera_back);
        if length(projected) > 0.0 { up = normalize(projected); }
        right = safe_normalize(cross(up, camera_back));
    }
    let cs = particle.size_spin.z;
    let sn = particle.size_spin.w;
    var side = (right * cs + up * sn) * particle.size_spin.x * 0.5;
    var vertical = (up * cs - right * sn) * particle.size_spin.y * 0.5;
    var cell = particle.cells.x;
    if tail {
        let extent = velocity * particle.velocity_tail.w;
        side = safe_normalize(cross(extent, view.world_position - center))
            * particle.size_spin.x * 0.5;
        vertical = extent * 0.5;
        center -= vertical;
        cell = particle.cells.y;
    }
    let world = center + side * input.corner.x + vertical * input.corner.y;
    let normal = safe_normalize(cross(side, vertical));
    let rows = max(emitter.atlas.x, 1u);
    let columns = max(emitter.atlas.y, 1u);
    cell %= rows * columns;
    var corner = (input.corner.xy + vec2(1.0)) * 0.5;
    if (emitter.atlas.z & 8u) != 0u { corner.y = 1.0 - corner.y; }
    var output: VertexOutput;
    output.position = position_world_to_clip(world);
    output.uv = (vec2(f32(cell % columns), f32(cell / columns)) + corner)
        / vec2(f32(columns), f32(rows));
    output.uv_secondary = particle.offsets.xy + corner * particle.scales.x;
    output.uv_tertiary = particle.offsets.zw + corner * particle.scales.y;
    output.color = particle.color;
    output.world_position = world;
    output.world_normal = normal;
    return output;
}
@fragment
fn fragment(input: VertexOutput, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
    let a = textureSample(primary, primary_sampler, input.uv);
    var color = a * input.color;
    var alpha_test = color.a;
    let multi = (emitter.atlas.z & 1u) != 0u;
    if multi {
        let b = textureSample(secondary, secondary_sampler, input.uv_secondary);
        let c = textureSample(tertiary, tertiary_sampler, input.uv_tertiary);
        let three_colors = (emitter.atlas.z & 2u) != 0u;
        var rgb = a.rgb * b.rgb;
        if three_colors { rgb *= c.rgb; }
        let rgb_gain = select(2.0, 4.0, three_colors);
        let alpha_gain = select(2.0, 4.0, (emitter.atlas.z & 4u) != 0u);
        alpha_test = a.a * b.a * c.a * alpha_gain;
        color = vec4(input.color.rgb * rgb * rgb_gain, input.color.a * alpha_test);
    }
#ifdef ALPHA_KEY
    if alpha_test < 128.0 / 255.0 { discard; }
    if !multi { color.a = 1.0; }
#endif
    if emitter.atlas.w == 0u { color.a = 1.0; }
    var fog_blend = 0u;
    if emitter.atlas.w == 3u || emitter.atlas.w == 4u { fog_blend = 1u; }
    if emitter.atlas.w == 5u { fog_blend = 2u; }
    if emitter.atlas.w == 6u { fog_blend = 3u; }
    return shade_effect(color, input.world_position, select(-input.world_normal, input.world_normal, front_facing), input.position,
        emitter.flags.z != 0u, emitter.flags.w != 0u, fog_blend);
}
