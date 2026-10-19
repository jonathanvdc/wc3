//! Surface-value evaluation independent of texture pixels and renderer materials.
use crate::model::{MeshId, Model, NodeFeature, Property, TextureTransform, ValueSample};
use glam::Vec4;
use glam::{Affine2, EulerRot, Mat4, Quat, Vec2, Vec3};
use std::collections::HashMap;
/// Prepared geometry color and opacity slots, reusable across samples.
#[derive(Clone, Debug)]
pub struct MeshAppearance {
    color: Option<Property<Vec3>>,
    opacity: Property<f32>,
    defaults: Vec4,
    animated: bool,
}
impl MeshAppearance {
    /// Compiles geometry appearance consumers without copying their values.
    pub fn compile(model: &Model<'_>) -> HashMap<MeshId, Self> {
        let mut values = HashMap::new();
        for (_, feature) in model.features() {
            if let NodeFeature::GeometryAnimation(source) = &feature.payload {
                if let Some(mesh) = source.mesh {
                    let color = source.use_color.then_some(source.color);
                    let rgb = color.map_or([1.; 3], |slot| {
                        (*model.values().get(slot).expect("validated slot")).to_array()
                    });
                    let opacity = *model.values().get(source.opacity).expect("validated slot");
                    values.insert(
                        mesh,
                        Self {
                            color,
                            opacity: source.opacity,
                            defaults: Vec4::from_array([rgb[0], rgb[1], rgb[2], opacity]),
                            animated: color.is_some_and(|slot| model.is_animated(slot))
                                || model.is_animated(source.opacity),
                        },
                    );
                }
            }
        }
        values
    }
    /// Authored opacity before sampling.
    pub fn default_opacity(&self) -> f32 {
        self.defaults[3]
    }
    /// Whether any program writes this appearance.
    pub fn is_animated(&self) -> bool {
        self.animated
    }
    /// Authored linear RGBA multiplier.
    pub fn defaults(&self) -> Vec4 {
        self.defaults
    }
    /// Reads the evaluated appearance slots.
    pub fn sample(&self, sample: &impl ValueSample) -> Vec4 {
        let rgb = self
            .color
            .map_or([1.; 3], |slot| (sample.animated(&slot)).to_array());
        Vec4::from_array([rgb[0], rgb[1], rgb[2], sample.animated(&self.opacity)])
    }
}
impl TextureTransform {
    /// Samples a 2D coordinate transform. Layered coordinates rotate around their center;
    /// model shader coordinates use their authored origin and Z Euler rotation.
    #[inline]
    pub fn affine(&self, sample: &impl ValueSample, centered: bool) -> Affine2 {
        let translation = sample.animated(&self.translation);
        let rotation = sample.animated(&self.rotation);
        let scale = sample.animated(&self.scale);
        if centered {
            Self::centered_affine(translation, rotation, scale)
        } else {
            let (_, _, z) = rotation.to_euler(EulerRot::XYZ);
            Affine2::from_scale_angle_translation(
                Vec2::new(scale[0], scale[1]),
                z,
                Vec2::new(translation[0], translation[1]),
            )
        }
    }
    /// Projects a complete texture transform around the layered shader's UV center.
    #[inline]
    pub fn centered_affine(translation: Vec3, rotation: Quat, scale: Vec3) -> Affine2 {
        let center = Vec3::new(0.5, 0.5, 0.);
        let matrix = Mat4::from_translation(translation + center)
            * Mat4::from_scale_rotation_translation(scale, rotation.normalize(), Vec3::ZERO)
            * Mat4::from_translation(-center);
        Affine2::from_cols(
            matrix.x_axis.truncate().truncate(),
            matrix.y_axis.truncate().truncate(),
            matrix.w_axis.truncate().truncate(),
        )
    }
}
/// Borrowed color inputs compiled from a native surface. No image assets are required.
#[derive(Default)]
pub struct SurfaceColor<'a> {
    /// RGB override, taking precedence over RGBA RGB.
    pub color: Option<&'a Property<Vec3>>,
    /// RGBA surface multiplier.
    pub rgba: Option<&'a Property<Vec4>>,
    /// Explicit opacity multiplier.
    pub opacity: Option<&'a Property<f32>>,
    /// Additional input weight.
    pub weight: Option<&'a Property<f32>>,
    /// Ordered pass contribution weights.
    pub pass_weights: &'a [Property<f32>],
    /// Authored untinted color.
    pub fallback: Vec4,
    /// Authored opacity before weight multiplication.
    pub unweighted_opacity: Option<f32>,
}
impl SurfaceColor<'_> {
    /// Samples linear RGBA and applies a geometry-owned tint.
    #[inline]
    pub fn sample(&self, sample: &impl ValueSample, tint: Vec4) -> Vec4 {
        let rgb = self.color.map(|v| sample.animated(v));
        let rgba = self.rgba.map(|v| sample.animated(v));
        let rgb = rgb
            .or(rgba.map(|v| v.truncate()))
            .unwrap_or(Vec3::from_array([
                self.fallback[0],
                self.fallback[1],
                self.fallback[2],
            ]));
        let alpha = self.opacity.map(|v| sample.animated(v)).unwrap_or_else(|| {
            if self.rgba.is_none() && self.weight.is_none() {
                self.unweighted_opacity.unwrap_or(self.fallback[3])
            } else {
                1.
            }
        }) * rgba.map_or(1., |v| v[3])
            * self.weight.map(|v| sample.animated(v)).unwrap_or(1.)
            * self
                .pass_weights
                .iter()
                .map(|v| sample.animated(v))
                .product::<f32>();
        Vec4::from_array([
            rgb[0] * tint[0],
            rgb[1] * tint[1],
            rgb[2] * tint[2],
            alpha * tint[3],
        ])
    }
}
