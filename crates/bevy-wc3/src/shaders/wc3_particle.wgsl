#import bevy_pbr::{
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
    pbr_types::pbr_input_new,
    pbr_functions::{apply_pbr_lighting, calculate_view},
}

@group(3) @binding(0) var particle_texture: texture_2d<f32>;
@group(3) @binding(1) var particle_sampler: sampler;

struct Particle {
    position_birth: vec4<f32>,
    velocity_gravity: vec4<f32>,
    lifetime_tail_birth: vec4<f32>,
    scale_facing: vec4<f32>,
};
struct Emitter {
    world_from_local: mat4x4<f32>,
    colors: array<vec4<f32>, 3>,
    scaling: vec4<f32>,
    intervals: array<vec4<u32>, 4>,
    atlas_flags: vec4<u32>,
    clock_tail: vec4<f32>,
    // x: Unshaded; remaining components reserved.
    render_flags: vec4<u32>,
};
@group(3) @binding(2) var<uniform> emitter: Emitter;
@group(3) @binding(3) var<storage, read> particles: array<Particle>;

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

    let acceleration = vec3(0.0, 0.0, -particle.velocity_gravity.w);
    var velocity = particle.velocity_gravity.xyz + acceleration * age;
    var center = particle.position_birth.xyz + particle.velocity_gravity.xyz * age
        + 0.5 * acceleration * age * age;
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
        side = vec3(cs, sn, 0.0) * half_size * size_scale;
        vertical = vec3(-sn, cs, 0.0) * half_size * size_scale;
        normal = vec3(0.0, 0.0, 1.0);
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
    if emitter.render_flags.x != 0u { return color; }
    // Use scene lighting with a matte, zero-reflectance material. This is
    // Bevy lighting, rather than the Classic clamped lighting equation.
    // Keep the texture/segment alpha unchanged by lighting.
    var lighting = pbr_input_new();
    lighting.material.base_color = color;
    lighting.material.reflectance = vec3(0.0);
    lighting.material.perceptual_roughness = 1.0;
    lighting.frag_coord = input.position;
    lighting.world_position = vec4(input.world_position, 1.0);
    lighting.world_normal = input.world_normal;
    lighting.N = input.world_normal;
    lighting.is_orthographic = view.clip_from_view[3].w == 1.0;
    lighting.V = calculate_view(lighting.world_position, lighting.is_orthographic);
    let shaded = apply_pbr_lighting(lighting);
    return vec4(shaded.rgb, color.a);
}
