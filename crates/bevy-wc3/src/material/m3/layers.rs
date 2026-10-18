//! Fixed standard-shader roles and owned animation evaluation.
use crate::animation_targets::AnimationTargets;
use bevy::math::{Affine2, Mat2, Mat3, UVec4, Vec2, Vec4};
use bevy::render::render_resource::ShaderType;
use wc3::model::materials::m3::{PropertyBinding, TextureLayer};
use wc3::model::{SampledAnimation, TrackValue, ValueSample};

pub(super) const ROLE_COUNT: usize = 15;
#[derive(Clone, Copy, Debug, PartialEq, ShaderType)]
pub(crate) struct LayerUniform {
    pub uv: Mat3,
    pub color: Vec4,
    pub controls: Vec4,
    // x: enabled, y: image present, z: channel, w: native layer flags.
    pub flags: UVec4,
    // x: raw UV stream index.
    pub route: UVec4,
}
impl Default for LayerUniform {
    fn default() -> Self {
        Self {
            uv: Mat3::IDENTITY,
            color: Vec4::ONE,
            controls: Vec4::new(1., 0., 1., 0.),
            flags: UVec4::ZERO,
            route: UVec4::ZERO,
        }
    }
}
#[derive(Clone, Debug, ShaderType)]
pub(crate) struct M3Uniform {
    // x: enabled, y: vertex RGB, z: vertex alpha, w: specular mode.
    pub settings: UVec4,
    pub blends: UVec4,
    pub diffuse_team: Vec4,
    pub emissive_team: Vec4,
    // x: exponent, y: specular HDR, z: emission HDR.
    pub response: Vec4,
    pub normal_weights: [Vec4; 2],
    pub layers: [LayerUniform; ROLE_COUNT],
}
impl Default for M3Uniform {
    fn default() -> Self {
        Self {
            settings: UVec4::ZERO,
            blends: UVec4::ZERO,
            diffuse_team: Vec4::ONE,
            emissive_team: Vec4::ONE,
            response: Vec4::ZERO,
            normal_weights: [Vec4::ZERO; 2],
            layers: [LayerUniform::default(); ROLE_COUNT],
        }
    }
}
#[derive(Clone)]
pub(crate) struct M3Animation {
    pub layers: [Option<TextureLayer>; ROLE_COUNT],
    pub weights: Vec<PropertyBinding<f32>>,
}
fn value<T: TrackValue>(property: &PropertyBinding<T>, sample: &impl ValueSample) -> T {
    sample.animated(&property.value)
}
pub(super) fn layer_uniform(
    layer: &TextureLayer,
    route: u32,
    sample: &impl ValueSample,
) -> LayerUniform {
    let binding = &layer.binding;
    let transform = &binding.coordinates.transform;
    let offset = Vec2::from_array(value(&transform.uv_offset, sample));
    let angle = value(&transform.uv_angle, sample)[2];
    let tiling = Vec2::from_array(value(&transform.uv_tiling, sample));
    let matrix = Mat2::from_diagonal(tiling) * Mat2::from_angle(angle);
    let uv = Affine2 {
        matrix2: matrix,
        translation: Vec2::splat(0.5) - matrix * (Vec2::splat(0.5) + offset),
    };
    LayerUniform {
        uv: Mat3::from(uv),
        color: Vec4::from_array(value(&binding.contribution.color, sample)),
        controls: Vec4::new(
            value(&binding.contribution.rgb_multiply, sample),
            value(&binding.contribution.rgb_add, sample),
            value(&binding.contribution.map_alpha, sample),
            0.,
        ),
        flags: UVec4::new(
            1,
            u32::from(binding.image.is_some()),
            binding.contribution.color_type.raw(),
            binding.sampling.flags.bits(),
        ),
        route: UVec4::new(route, 0, 0, 0),
    }
}
pub(super) fn animated_layer(
    layer: &TextureLayer,
    targets: &AnimationTargets,
) -> Option<TextureLayer> {
    let c = &layer.binding.contribution;
    let t = &layer.binding.coordinates.transform;
    let animated = targets.contains(&c.color.value)
        || targets.scalar(&c.map_alpha.value)
        || targets.scalar(&c.rgb_multiply.value)
        || targets.scalar(&c.rgb_add.value)
        || targets.contains(&t.uv_offset.value)
        || targets.contains(&t.uv_angle.value)
        || targets.contains(&t.uv_tiling.value);
    animated.then(|| layer.clone())
}

impl M3Animation {
    pub(crate) fn is_animated(&self) -> bool {
        self.layers.iter().any(Option::is_some) || !self.weights.is_empty()
    }
    pub(crate) fn apply(&self, sample: &SampledAnimation<'_, '_>, target: &mut M3Uniform) -> bool {
        let mut changed = false;
        for (slot, layer) in self.layers.iter().enumerate() {
            if let Some(layer) = layer {
                let mut next = layer_uniform(layer, target.layers[slot].route.x, sample);
                // Image resolution and role availability are compile-time choices.
                next.flags.x = target.layers[slot].flags.x;
                next.flags.y = target.layers[slot].flags.y;
                next.route.y = target.layers[slot].route.y;
                if target.layers[slot] != next {
                    target.layers[slot] = next;
                    changed = true;
                }
            }
        }
        for (index, weight) in self.weights.iter().take(8).enumerate() {
            let next = value(weight, sample);
            if target.normal_weights[index / 4][index % 4] != next {
                target.normal_weights[index / 4][index % 4] = next;
                changed = true;
            }
        }
        changed
    }
}
