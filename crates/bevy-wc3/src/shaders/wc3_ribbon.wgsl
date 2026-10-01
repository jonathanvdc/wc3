#import bevy_pbr::{
    mesh_view_bindings::view,
    view_transformations::position_world_to_clip,
    pbr_types::pbr_input_new,
    pbr_functions::{apply_pbr_lighting, calculate_view},
}

@group(3) @binding(0) var ribbon_texture: texture_2d<f32>;
@group(3) @binding(1) var ribbon_sampler: sampler;

struct Section {
    above_birth: vec4<f32>,
    below_birth: vec4<f32>,
};
struct Emitter {
    color: vec4<f32>,
    clock_gravity: vec4<f32>,
    atlas_flags: vec4<u32>,
    uv_transform: mat4x4<f32>,
};
@group(3) @binding(2) var<uniform> emitter: Emitter;
@group(3) @binding(3) var<storage, read> sections: array<Section>;

struct VertexInput {
    @location(0) corner: vec3<f32>,
    // Older slot, newer slot, chronological segment rank, chain segment count.
    @location(3) segment: vec4<u32>,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
};
fn displacement(section: Section) -> vec3<f32> {
    let age = max((emitter.clock_gravity.x - section.above_birth.w)
        + (emitter.clock_gravity.y - section.below_birth.w), 0.0);
    return vec3(0.0, 0.0, -0.5 * emitter.clock_gravity.z * age * age);
}
@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    let older = sections[input.segment.x];
    let newer = sections[input.segment.y];
    let old_above = older.above_birth.xyz + displacement(older);
    let old_below = older.below_birth.xyz + displacement(older);
    let new_above = newer.above_birth.xyz + displacement(newer);
    let new_below = newer.below_birth.xyz + displacement(newer);
    let horizontal = (input.corner.x + 1.0) * 0.5;
    let vertical = (input.corner.y + 1.0) * 0.5;
    let above = mix(old_above, new_above, horizontal);
    let below = mix(old_below, new_below, horizontal);
    let world = mix(below, above, vertical);
    let axis = new_above + new_below - old_above - old_below;
    let width = above - below;
    let cross_normal = cross(axis, width);
    var normal = vec3(0.0, 0.0, 1.0);
    if length(cross_normal) > 0.00001 { normal = normalize(cross_normal); }
    let rows = emitter.atlas_flags.x;
    let columns = emitter.atlas_flags.y;
    let slot = emitter.atlas_flags.z;
    // One atlas cell spans each connected live chain, newest end at U=0.
    let u = 1.0 - (f32(input.segment.z) + horizontal) / f32(max(input.segment.w, 1u));
    let uv = vec2((f32(slot % columns) + u) / f32(columns),
        (f32(slot / columns) + 1.0 - vertical) / f32(rows));
    var output: VertexOutput;
    output.position = position_world_to_clip(world);
    output.uv = (emitter.uv_transform * vec4(uv, 0.0, 1.0)).xy;
    output.color = emitter.color;
    output.world_position = world;
    output.world_normal = normal;
    return output;
}
@fragment
fn fragment(input: VertexOutput, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
    // Zero-alpha ribbons must also disappear with modulation blend modes.
    if input.color.a <= 0.0 { discard; }
    let color = textureSample(ribbon_texture, ribbon_sampler, input.uv) * input.color;
#ifdef ALPHA_KEY
    if color.a < 0.5 { discard; }
#endif
    if emitter.atlas_flags.w != 0u { return color; }
    // Use Bevy scene lighting with a matte, zero-reflectance material.
    // Keep the texture/segment alpha unchanged by lighting.
    var lighting = pbr_input_new();
    lighting.material.base_color = color;
    lighting.material.reflectance = vec3(0.0);
    lighting.material.perceptual_roughness = 1.0;
    lighting.frag_coord = input.position;
    lighting.world_position = vec4(input.world_position, 1.0);
    lighting.world_normal = select(-input.world_normal, input.world_normal, front_facing);
    lighting.N = lighting.world_normal;
    lighting.is_orthographic = view.clip_from_view[3].w == 1.0;
    lighting.V = calculate_view(lighting.world_position, lighting.is_orthographic);
    let shaded = apply_pbr_lighting(lighting);
    return vec4(shaded.rgb, color.a);
}
