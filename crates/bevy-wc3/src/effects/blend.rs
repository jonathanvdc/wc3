//! Render operations shared by immutable sprite and ribbon transports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum EffectBlend {
    Opaque,
    Cutout,
    Alpha,
    Additive,
    AdditiveAlpha,
    Modulate,
    Modulate2x,
}

impl EffectBlend {
    /// Target contribution at full fog: scene color, zero, one, or one half.
    pub(crate) fn fog_blend(self) -> u32 {
        match self {
            Self::Opaque | Self::Cutout | Self::Alpha => 0,
            Self::Additive | Self::AdditiveAlpha => 1,
            Self::Modulate => 2,
            Self::Modulate2x => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use naga::front::wgsl::parse_str;
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    #[test]
    fn effect_lighting_validates_with_and_without_distance_fog() {
        let shader = include_str!("../shaders/effect_lighting.wgsl");
        let body = &shader[shader.find("// Procedural").unwrap()..];
        let imports = r#"
struct View { clip_from_view: mat4x4<f32>, world_position: vec4<f32> };
@group(0) @binding(0) var<uniform> view: View;
struct Fog { base_color: vec4<f32>, directional_light_color: vec4<f32> };
@group(0) @binding(1) var<uniform> fog: Fog;
struct Material { base_color: vec4<f32>, reflectance: vec3<f32>, perceptual_roughness: f32, flags: u32 };
struct Lighting {
    material: Material, frag_coord: vec4<f32>, world_position: vec4<f32>,
    world_normal: vec3<f32>, N: vec3<f32>, V: vec3<f32>, is_orthographic: bool, flags: u32,
};
const MESH_FLAGS_SHADOW_RECEIVER_BIT: u32 = 1u << 29u;
fn pbr_input_new() -> Lighting { var result: Lighting; return result; }
fn calculate_view(p: vec4<f32>, orthographic: bool) -> vec3<f32> { return vec3(0.0, 0.0, 1.0); }
fn apply_pbr_lighting(p: Lighting) -> vec4<f32> { return p.material.base_color; }
fn apply_fog(f: Fog, c: vec4<f32>, p: vec3<f32>, v: vec3<f32>, xy: vec2<f32>) -> vec4<f32> { return c; }
fn main_pass_post_lighting_processing(p: Lighting, c: vec4<f32>) -> vec4<f32> { return c; }
"#;
        for fog in [false, true] {
            let mut source = imports.to_owned();
            let mut enabled = true;
            for line in body.lines() {
                match line.trim() {
                    "#ifdef DISTANCE_FOG" => enabled = fog,
                    "#endif" => enabled = true,
                    _ if enabled => {
                        source.push_str(line);
                        source.push('\n');
                    }
                    _ => {}
                }
            }
            let module = parse_str(&source)
                .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
            Validator::new(ValidationFlags::all(), Capabilities::all())
                .validate(&module)
                .unwrap();
        }
    }
}
