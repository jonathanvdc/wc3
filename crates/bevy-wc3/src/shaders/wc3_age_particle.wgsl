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
#ifdef REFRACTION
struct RefractionView {
    projection_scale: vec2<f32>,
    strength: f32,
    fade_distance_inverse: f32,
};
@group(3) @binding(0) var<uniform> refraction_view: RefractionView;
#endif
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
#ifdef SAMPLED_SPRITE
    @location(6) sampled_appearance: vec2<f32>,
#endif
};
fn safe_normalize(value: vec3<f32>) -> vec3<f32> {
    let magnitude = length(value);
    if magnitude == 0.0 { return vec3(0.0); }
    return value / magnitude;
}
#ifdef SAMPLED_SPRITE
fn normalize_or(value: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let scale = max(max(abs(value.x), abs(value.y)), abs(value.z));
    if scale == 0.0 { return fallback; }
    return normalize(value / scale);
}
fn sampled_axes(particle: Particle, camera_right: vec3<f32>, camera_up: vec3<f32>,
    camera_back: vec3<f32>) -> mat2x3<f32> {
    let mode = emitter.flags.x;
    let has_axis0 = (emitter.flags.y & 1u) != 0u;
    let has_axis1 = (emitter.flags.y & 2u) != 0u;
    let axis0 = particle.velocity_tail.xyz;
    let axis1 = particle.offsets.xyz;
    let size = particle.size_spin.x * 0.5;
    var right = camera_right * size;
    var up = camera_up * size;
    if mode == 0u && has_axis0 && has_axis1 {
        right = axis0 * size;
        up = axis1 * size;
    }
    if mode == 2u && has_axis0 {
        let direction = normalize_or(particle.position.xyz - view.world_position, -camera_back);
        right = normalize_or(cross(direction, normalize_or(axis0, vec3(0.0))), camera_right) * size;
        return mat2x3(right, axis0 * 0.5);
    }
    if mode == 3u && has_axis0 {
        let direction = normalize_or(particle.position.xyz - view.world_position, -camera_back);
        let width = normalize_or(cross(normalize_or(axis0, vec3(0.0)), direction), camera_right) * size;
        return mat2x3(-width, axis0 * 0.5 + cross(direction, width));
    }
    if mode == 2u || mode == 3u || mode == 4u { return mat2x3(right, up); }
    if mode == 5u && has_axis0 && has_axis1 {
        var normal = normalize_or(cross(normalize_or(axis0, vec3(0.0)),
            normalize_or(axis1, vec3(0.0))), vec3(0.0));
        if length(normal) == 0.0 {
            let axis = normalize_or(axis1, vec3(0.0));
            var helper = vec3(1.0, 0.0, 0.0);
            if abs(axis.x) > 0.9 { helper = vec3(0.0, 1.0, 0.0); }
            normal = safe_normalize(cross(axis, helper));
        }
        right = normal * size;
        up = cross(axis1, normal) * size;
    }
    let cs = particle.size_spin.z;
    var sn = particle.size_spin.w;
    if mode == 5u { sn = -sn; }
    return mat2x3(right * cs + up * sn, up * cs - right * sn);
}
#endif
#ifdef SAMPLED_RIBBON
// Ribbon fields are packed by RibbonSegmentParameters; positions remain exact
// endpoint samples so adjacent segments reuse identical world cross sections.
fn ribbon_vertex(input: VertexInput) -> VertexOutput {
    let segment = particles[input.particle_index & 0x7fffffffu];
    let endpoint = input.corner.y > 0.0;
    let position = select(segment.position.xyz, segment.velocity_tail.xyz, endpoint);
    let half_width = select(segment.position.w, segment.velocity_tail.w, endpoint);
    let section = select(segment.size_spin.xyz, segment.offsets.xyz, endpoint);
    var across = section;
    if emitter.flags.x == 0u {
        across = normalize_or(cross(section, normalize_or(position - view.world_position, vec3(0.0))), vec3(1.0, 0.0, 0.0));
    }
    let world = position + across * (half_width * input.corner.x);
    let normal = normalize_or(cross(across, normalize_or(segment.velocity_tail.xyz - segment.position.xyz, vec3(0.0))), vec3(0.0, 0.0, 1.0));
    var uv = vec2(select(segment.size_spin.w, segment.offsets.w, endpoint), (1.0 - input.corner.x) * 0.5);
    // Canonical ribbon UVs already have V0 at +cross-section; no image flip.
    if (emitter.atlas.z & 256u) != 0u { uv = vec2(uv.y, 1.0 - uv.x); }
    if (emitter.atlas.z & 512u) != 0u { uv.x = 1.0 - uv.x; }
    if (emitter.atlas.z & 1024u) != 0u { uv.y = 1.0 - uv.y; }
    let rows = max(emitter.atlas.x, 1u);
    let columns = max(emitter.atlas.y, 1u);
    let cell = segment.cells.x % (rows * columns);
    var output: VertexOutput;
    output.position = position_world_to_clip(world);
    output.uv = (vec2(f32(cell % columns), f32(cell / columns)) + uv) / vec2(f32(columns), f32(rows));
    output.color = select(segment.color, segment.scales, endpoint);
    output.world_position = world;
    output.world_normal = normal;
    output.uv_secondary = vec2(0.0);
    output.uv_tertiary = vec2(0.0);
    output.sampled_appearance = vec2(0.0, select(bitcast<f32>(segment.cells.y), bitcast<f32>(segment.cells.z), endpoint));
    return output;
}
#endif
@vertex
fn vertex(input: VertexInput) -> VertexOutput {
#ifdef SAMPLED_RIBBON
    return ribbon_vertex(input);
#else
    let particle = particles[input.particle_index & 0x7fffffffu];
    let tail = (input.particle_index & 0x80000000u) != 0u;
    let camera_back = safe_normalize(view.world_from_view[2].xyz);
    var right = safe_normalize(view.world_from_view[0].xyz);
    var up = safe_normalize(view.world_from_view[1].xyz);
    var center = particle.position.xyz;
    let velocity = particle.velocity_tail.xyz;
    var side: vec3<f32>;
    var vertical: vec3<f32>;
#ifdef SAMPLED_SPRITE
    let axes = sampled_axes(particle, right, up, camera_back);
    side = axes[0];
    vertical = axes[1];
#else
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
    side = (right * cs + up * sn) * particle.size_spin.x;
    vertical = (up * cs - right * sn) * particle.size_spin.y;
#endif
    var cell = particle.cells.x;
    if tail {
        let extent = velocity * particle.velocity_tail.w;
        side = safe_normalize(cross(extent, view.world_position - center))
            * particle.size_spin.x;
        vertical = extent * 0.5;
        center -= vertical;
        cell = particle.cells.y;
    }
    var world = center + side * input.corner.x + vertical * input.corner.y;
    var normal = safe_normalize(cross(side, vertical));
#ifdef SAMPLED_SPRITE
    let capsule = emitter.flags.x == 4u && (emitter.flags.y & 1u) != 0u;
    if capsule {
        let axis = particle.velocity_tail.xyz;
        let direction = normalize_or(center - view.world_position, -camera_back);
        let width = normalize_or(cross(normalize_or(axis, vec3(0.0)), direction), right)
            * particle.size_spin.x * 0.5;
        let padding = cross(direction, width);
        world = center + axis * (input.corner.y * 0.5) + width * input.corner.x;
        normal = normalize_or(cross(normalize_or(axis, vec3(0.0)),
            normalize_or(width, vec3(0.0))), camera_back);
        if abs(input.corner.z) > 0.0 {
            normal = normalize_or(cross(normalize_or(padding, vec3(0.0)),
                normalize_or(width, vec3(0.0))), camera_back);
        }
        if abs(input.corner.z) == 2.0 {
            world = center + (axis * 0.5 + padding) * input.corner.y;
        }
    }
#endif
    let rows = max(emitter.atlas.x, 1u);
    let columns = max(emitter.atlas.y, 1u);
    cell %= rows * columns;
    var corner = (input.corner.xy + vec2(1.0)) * 0.5;
    if (emitter.atlas.z & 8u) != 0u { corner.y = 1.0 - corner.y; }
#ifdef SAMPLED_SPRITE
    if capsule {
        corner = vec2((input.corner.x + 1.0) * 0.5);
        if abs(input.corner.z) == 2.0 {
            corner = select(vec2(0.0, 1.0), vec2(1.0, 0.0), input.corner.y > 0.0);
        }
    }
    if (emitter.atlas.z & 256u) != 0u { corner = vec2(corner.y, 1.0 - corner.x); }
    if (emitter.atlas.z & 512u) != 0u { corner.x = 1.0 - corner.x; }
    if (emitter.atlas.z & 1024u) != 0u { corner.y = 1.0 - corner.y; }
#endif
    var output: VertexOutput;
    output.position = position_world_to_clip(world);
    output.uv = (vec2(f32(cell % columns), f32(cell / columns)) + corner)
        / vec2(f32(columns), f32(rows));
    output.uv_secondary = particle.offsets.xy + corner * particle.scales.x;
    output.uv_tertiary = particle.offsets.zw + corner * particle.scales.y;
#ifdef SAMPLED_SPRITE
    let next = particle.cells.z % (rows * columns);
    output.uv_secondary = (vec2(f32(next % columns), f32(next / columns)) + corner)
        / vec2(f32(columns), f32(rows));
    output.sampled_appearance = particle.scales.zw;
#endif
    output.color = particle.color;
    output.world_position = world;
    output.world_normal = normal;
    return output;
#endif
}
@fragment
fn fragment(input: VertexOutput, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
    var a = textureSample(primary, primary_sampler, input.uv);
#ifdef REFRACTION
    let b = textureSample(primary, primary_sampler, input.uv_secondary);
    let c = textureSample(primary, primary_sampler, input.uv_tertiary);
    let depth = abs((view.view_from_world * vec4(input.world_position, 1.0)).z);
    let fade = 1.0 - clamp(depth * refraction_view.fade_distance_inverse, 0.0, 1.0);
    let mask = a.r * b.g * c.b * 4.0 * fade;
    return vec4(mask * input.color.r * 0.5, 0.0, 0.0, mask * input.color.a);
#else
#ifdef SAMPLED_SPRITE
    if (emitter.atlas.z & 64u) != 0u {
        let next = textureSample(primary, primary_sampler, input.uv_secondary);
        a = mix(a, next, input.sampled_appearance.x);
    }
#endif
    var color = a * input.color;
#ifdef SAMPLED_SPRITE
    if (emitter.atlas.z & 128u) != 0u {
        color.a = textureSample(secondary, secondary_sampler, vec2(color.a, input.sampled_appearance.y)).r;
    }
#endif
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
#endif
}
