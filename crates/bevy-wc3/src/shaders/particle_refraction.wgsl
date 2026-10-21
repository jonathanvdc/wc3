#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct RefractionView {
    projection_scale: vec2<f32>,
    strength: f32,
    fade_distance_inverse: f32,
};
@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var mask: texture_2d<f32>;
@group(0) @binding(3) var<uniform> settings: RefractionView;

fn remapped_height(pixel: vec2<i32>) -> f32 {
    let dimensions = vec2<i32>(textureDimensions(mask));
    let h = textureLoad(mask, clamp(pixel, vec2(0), dimensions - vec2(1)), 0).r;
    if h <= 0.5 { return h * 2.0; }
    return 0.5 / h;
}

@fragment
fn fragment(input: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(scene));
    let uv = input.position.xy / dimensions;
    let original = textureSampleLevel(scene, linear_sampler, uv, 0.0);
    if settings.strength == 0.0 { return original; }
    let pixel = vec2<i32>(input.position.xy);
    let heights = vec4<f32>(
        remapped_height(pixel + vec2(-1, 0)),
        remapped_height(pixel + vec2(1, 0)),
        remapped_height(pixel + vec2(0, -1)),
        remapped_height(pixel + vec2(0, 1)),
    );
    if any(heights == vec4(0.0)) { return original; }
    let normal = normalize(vec3(heights.x - heights.y, heights.w - heights.z, 1.0));
    let ray = refract(vec3(0.0, 0.0, -1.0), normal, 1.0 / 1.33);
    if ray.z == 0.0 { return original; }
    let offset = ray.xy / (-ray.z) * settings.projection_scale * settings.strength;
    let displaced = uv + offset;
    if textureSampleLevel(mask, linear_sampler, displaced, 0.0).r <= 0.0 {
        return original;
    }
    return textureSampleLevel(scene, linear_sampler, displaced, 0.0);
}
