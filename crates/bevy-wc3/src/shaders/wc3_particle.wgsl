#import bevy_pbr::{
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
}
#import "embedded://bevy_wc3/shaders/effect_lighting.wgsl"::shade_effect

@group(2) @binding(0) var particle_texture: texture_2d<f32>;
@group(2) @binding(1) var particle_sampler: sampler;

struct Particle {
    position_birth: vec4<f32>,
    velocity_gravity: vec4<f32>,
    lifetime_tail_birth: vec4<f32>,
    scale_facing: vec4<f32>,
    acceleration_drag: vec4<f32>,
};
struct Emitter {
    world_from_local: mat4x4<f32>,
    colors: array<vec4<f32>, 3>,
    scaling: vec4<f32>,
    intervals: array<vec4<u32>, 4>,
    atlas_flags: vec4<u32>,
    clock_tail: vec4<f32>,
    // x: Unshaded; y: fog enabled; remaining components reserved.
    render_flags: vec4<u32>,
    plane_axes: array<vec4<f32>, 2>,
};
@group(2) @binding(2) var<uniform> emitter: Emitter;
@group(2) @binding(3) var<storage, read> particles: array<Particle>;

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
};
fn safe_normalize(value: vec3<f32>) -> vec3<f32> {
    let length = length(value);
    if length < 0.00001 { return vec3(0.0, 0.0, 0.0); }
    return value / length;
}
@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    let particle = particles[input.particle_index];
    let age = max((emitter.clock_tail.x - particle.position_birth.w)
        + (emitter.clock_tail.y - particle.lifetime_tail_birth.z), 0.0);
    let life = clamp(age / particle.lifetime_tail_birth.x, 0.0, 1.0);
    let middle = emitter.scaling.w;
    var phase = 0u;
    var factor = life / middle;
    if life >= middle {
        phase = 1u;
        factor = (life - middle) / (1.0 - middle);
    }
    let color = mix(emitter.colors[phase], emitter.colors[phase + 1u], factor);
    let scale = mix(emitter.scaling[phase], emitter.scaling[phase + 1u], factor);
    let tail = particle.lifetime_tail_birth.y > 0.5;
    let interval = emitter.intervals[phase + select(0u, 2u, tail)];
    let rows = emitter.atlas_flags.x;
    let columns = emitter.atlas_flags.y;
    let count = interval.y - min(interval.x, interval.y);
    var cell = interval.x;
    if count > 0u {
        let offset = u32(floor(f32(count) * f32(interval.z) * factor)) % count;
        cell += min(offset, 0xffffffffu - cell);
    }
    var cells = 0xffffffffu;
    if columns <= 0xffffffffu / rows {
        cells = rows * columns;
    }
    cell = min(cell, cells - 1u);
    let uv_origin = vec2(f32(cell % columns) / f32(columns), f32(cell / columns) / f32(rows));
    let uv_size = vec2(1.0 / f32(columns), 1.0 / f32(rows));

    let acceleration = particle.acceleration_drag.xyz;
    let drag = particle.acceleration_drag.w;
    var velocity = particle.velocity_gravity.xyz + acceleration * age;
    var center = particle.position_birth.xyz + particle.velocity_gravity.xyz * age
        + 0.5 * acceleration * age * age;
    if drag > 0.00001 {
        let decay = exp(-drag * age);
        let velocity_time = (1.0 - decay) / drag;
        velocity = particle.velocity_gravity.xyz * decay + acceleration * velocity_time;
        center = particle.position_birth.xyz + particle.velocity_gravity.xyz * velocity_time
            + acceleration * ((age - velocity_time) / drag);
    }
    if emitter.atlas_flags.z != 0u {
        center = (emitter.world_from_local * vec4(center, 1.0)).xyz;
        velocity = (emitter.world_from_local * vec4(velocity, 0.0)).xyz;
    }
    let right = view.world_from_view[0].xyz;
    let up = view.world_from_view[1].xyz;
    let forward = -view.world_from_view[2].xyz;
    let half_size = max(scale, 0.0) * 0.5;
    let size_scale = particle.scale_facing.xyz;
    // Billboard heads and tails use the camera-facing lighting normal;
    // XYQuad heads override it with world +Z.
    var normal = -forward;
    var side = right * half_size;
    var vertical = up * half_size;
    if tail {
        let axis = safe_normalize(velocity);
        side = safe_normalize(cross(axis, forward)) * half_size * size_scale;
        // Velocity already contains the motion scale. Scale only the width
        // here; scaling the tail length again would apply that scale twice.
        vertical = velocity * emitter.clock_tail.z * 0.5;
        center -= vertical;
    } else if emitter.atlas_flags.w != 0u {
        // XYQuad stays in world XY; gravity and ModelSpace motion do not rotate it.
        let facing = particle.scale_facing.w;
        let cs = cos(facing);
        let sn = sin(facing);
        let axis_x = emitter.plane_axes[0].xyz;
        let axis_y = emitter.plane_axes[1].xyz;
        side = (axis_x * cs + axis_y * sn) * half_size * size_scale;
        vertical = (-axis_x * sn + axis_y * cs) * half_size * size_scale;
        normal = safe_normalize(cross(axis_x, axis_y));
    } else {
        // Scale world components after orienting the billboard.
        side *= size_scale;
        vertical *= size_scale;
    }
    let world = center + side * input.corner.x + vertical * input.corner.y;
    var output: VertexOutput;
    output.position = position_world_to_clip(world);
    output.uv = uv_origin + vec2((input.corner.x + 1.0) * 0.5, (1.0 - input.corner.y) * 0.5) * uv_size;
    output.color = color;
    output.world_position = world;
    output.world_normal = normal;
    return output;
}
@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // Zero-alpha particles must also disappear with modulation blend modes.
    if input.color.a <= 0.0 { discard; }
    let color = textureSample(particle_texture, particle_sampler, input.uv) * input.color;
#ifdef ALPHA_KEY
    if color.a < 0.5 { discard; }
#endif
    return shade_effect(color, input.world_position, input.world_normal, input.position,
        emitter.render_flags.x != 0u, emitter.render_flags.y != 0u, emitter.render_flags.z);
}
