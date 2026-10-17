//! Surface-value evaluation independent of texture pixels and renderer materials.
use crate::model::materials::m2::{MaterialStep as M2MaterialStep, TextureUnit};
use crate::model::materials::mdlx::{
    Layer, LayeredSurface, MaterialInputs, MaterialStep as MdlxMaterialStep,
};
use crate::model::materials::program::{MaterialStep, MaterialSurface};
use crate::model::{
    Animated, BindingValue, ChannelValue, MeshId, Model, NodeFeature, Property, PropertySample,
    PropertyTarget, TextureTransform,
};
use glam::{Affine2, EulerRot, Mat4, Quat, Vec2, Vec3};
use std::collections::{HashMap, HashSet};
/// Prepared geometry color and opacity controls, reusable across samples.
#[derive(Clone, Debug)]
pub struct MeshAppearance {
    color: Animated<[f32; 3]>,
    opacity: Animated<f32>,
    common_color: Option<Property<[f32; 3]>>,
    common_opacity: Option<Property<f32>>,
    animated: bool,
}
impl MeshAppearance {
    /// Prepares geometry appearance controls for all meshes. Rebuild after changing
    /// geometry-animation definitions or channel targets; sample using the same model identities.
    pub fn compile(model: &Model<'_>) -> HashMap<MeshId, Self> {
        let targets: HashSet<PropertyTarget> = model
            .timeline_channels()
            .iter()
            .chain(model.shared_channels())
            .chain(model.clips().iter().flat_map(|(_, clip)| clip.channels()))
            .chain(
                model
                    .controllers()
                    .iter()
                    .flat_map(|(_, controller)| &controller.channels),
            )
            .map(|channel| channel.target().clone())
            .collect();
        let mut values = HashMap::new();
        for (_, feature) in model.features() {
            if let NodeFeature::GeometryAnimation(source) = &feature.payload {
                if let Some(mesh) = source.mesh {
                    let color = if source.use_color {
                        source.color.clone()
                    } else {
                        Animated::constant([1.; 3])
                    };
                    let opacity = source.opacity.clone();
                    let animated = color
                        .binding
                        .as_ref()
                        .is_some_and(|property| targets.contains(property.target()))
                        || opacity
                            .binding
                            .as_ref()
                            .is_some_and(|property| targets.contains(property.target()));
                    values.insert(
                        mesh,
                        Self {
                            color,
                            opacity,
                            common_color: None,
                            common_opacity: None,
                            animated,
                        },
                    );
                }
            }
        }
        for (mesh, _) in model.meshes() {
            let color = mesh.color();
            let opacity = mesh.opacity();
            let common_color = targets.contains(color.target()).then_some(color);
            let common_opacity = targets.contains(opacity.target()).then_some(opacity);
            if common_color.is_some() || common_opacity.is_some() {
                let value = values.entry(mesh).or_insert_with(|| Self {
                    color: Animated::constant([1.; 3]),
                    opacity: Animated::constant(1.),
                    common_color: None,
                    common_opacity: None,
                    animated: false,
                });
                value.common_color = common_color;
                value.common_opacity = common_opacity;
                value.animated = true;
            }
        }
        values
    }
    /// Authored opacity before sampling.
    pub fn default_opacity(&self) -> f32 {
        self.opacity.default
    }
    /// Whether any programme changes this appearance.
    pub fn is_animated(&self) -> bool {
        self.animated
    }
    /// Authored linear RGBA multiplier.
    pub fn defaults(&self) -> [f32; 4] {
        [
            self.color.default[0],
            self.color.default[1],
            self.color.default[2],
            self.opacity.default,
        ]
    }
    /// Samples linear RGBA, including common mesh overrides.
    #[inline]
    pub fn sample(&self, sample: &impl PropertySample) -> [f32; 4] {
        let color = self
            .common_color
            .as_ref()
            .and_then(|property| sample.property(property))
            .unwrap_or_else(|| sample.animated(&self.color));
        let opacity = self
            .common_opacity
            .as_ref()
            .and_then(|property| sample.property(property))
            .unwrap_or_else(|| sample.animated(&self.opacity));
        [color[0], color[1], color[2], opacity]
    }
}

impl<T: ChannelValue> Animated<T> {
    /// Freezes this value at the supplied sample, removing its property binding.
    pub fn sampled(&self, sample: &impl PropertySample) -> Self {
        Self::constant(sample.animated(self))
    }
}
impl TextureTransform {
    /// Freezes every component without imposing a UV projection convention.
    pub fn sampled(&self, sample: &impl PropertySample) -> Self {
        Self {
            translation: self.translation.sampled(sample),
            rotation: self.rotation.sampled(sample),
            scale: self.scale.sampled(sample),
        }
    }
    /// Samples a 2D coordinate transform. Layered coordinates rotate around their center;
    /// model shader coordinates use their authored origin and Z Euler rotation.
    #[inline]
    pub fn affine(&self, sample: &impl PropertySample, centered: bool) -> Affine2 {
        let translation = sample.animated(&self.translation);
        let rotation = sample.animated(&self.rotation).0;
        let scale = sample.animated(&self.scale);
        if centered {
            Self::centered_affine(translation, rotation, scale)
        } else {
            let (_, _, z) = Quat::from_array(rotation).to_euler(EulerRot::XYZ);
            Affine2::from_scale_angle_translation(
                Vec2::new(scale[0], scale[1]),
                z,
                Vec2::new(translation[0], translation[1]),
            )
        }
    }
    /// Projects a complete texture transform around the layered shader's UV center.
    #[inline]
    pub fn centered_affine(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> Affine2 {
        let center = Vec3::new(0.5, 0.5, 0.);
        let matrix = Mat4::from_translation(Vec3::from_array(translation) + center)
            * Mat4::from_scale_rotation_translation(
                Vec3::from_array(scale),
                Quat::from_array(rotation).normalize(),
                Vec3::ZERO,
            )
            * Mat4::from_translation(-center);
        Affine2::from_cols(
            matrix.x_axis.truncate().truncate(),
            matrix.y_axis.truncate().truncate(),
            matrix.w_axis.truncate().truncate(),
        )
    }
}
impl Layer {
    /// Freezes animated pass values and input-owned transforms while preserving render state.
    pub fn sampled(&self, sample: &impl PropertySample) -> Self {
        let mut value = self.clone();
        value.binding.image = self.binding.image.sampled(sample);
        value.binding.coordinates.transform = self.binding.coordinates.transform.sampled(sample);
        value.opacity = self.opacity.sampled(sample);
        value.emissive_gain = self.emissive_gain.as_ref().map(|v| v.sampled(sample));
        if let Some(fresnel) = &mut value.fresnel {
            fresnel.color = fresnel.color.sampled(sample);
            fresnel.opacity = fresnel.opacity.sampled(sample);
            fresnel.team_color = fresnel.team_color.sampled(sample);
        }
        for slot in &mut value.textures {
            slot.binding.image = slot.binding.image.sampled(sample);
        }
        value
    }
}
impl LayeredSurface<'_> {
    /// Freezes ordered steps without converting native layered shading into PBR.
    pub fn sampled(&self, sample: &impl PropertySample) -> Self {
        let mut value = self.clone();
        value.layers = self
            .layers
            .iter()
            .map(|layer| layer.sampled(sample))
            .collect();
        value
    }
}
impl MaterialInputs {
    /// Freezes shared WC3 shader inputs while retaining their draw state.
    pub fn sampled(&self, sample: &impl PropertySample) -> Self {
        let mut value = self.clone();
        value.binding.image = self.binding.image.sampled(sample);
        value.binding.coordinates.transform = self.binding.coordinates.transform.sampled(sample);
        value.opacity = self.opacity.sampled(sample);
        value.emissive_gain = self.emissive_gain.as_ref().map(|v| v.sampled(sample));
        value
    }
}
impl MdlxMaterialStep<'_> {
    /// Freezes shader inputs while preserving this step's shader and rendering settings.
    pub fn sampled(&self, sample: &impl PropertySample) -> Self {
        let mut value = self.clone();
        match &mut value {
            Self::Classic(step) => step.inputs = step.inputs.sampled(sample),
            Self::ReforgedDefault(step) => {
                step.inputs = step.inputs.sampled(sample);
                for slot in &mut step.textures {
                    slot.binding.image = slot.binding.image.sampled(sample);
                }
                if let Some(fresnel) = &mut step.fresnel {
                    fresnel.color = fresnel.color.sampled(sample);
                    fresnel.opacity = fresnel.opacity.sampled(sample);
                    fresnel.team_color = fresnel.team_color.sampled(sample);
                }
            }
            Self::Unresolved(step) => step.layer = step.layer.sampled(sample),
        }
        value
    }
}
impl MaterialSurface<'_> {
    /// Freezes layered and model-shader inputs while retaining order and pass settings.
    ///
    /// Other typed shader programmes and composite references are cloned unchanged;
    /// this helper does not execute or flatten them.
    pub fn sampled(&self, sample: &impl PropertySample) -> Self {
        Self {
            steps: self
                .steps
                .iter()
                .map(|pass| match pass {
                    MaterialStep::Mdlx(value) => {
                        MaterialStep::Mdlx(Box::new(value.sampled(sample)))
                    }
                    MaterialStep::M2(value) => MaterialStep::M2(Box::new(value.sampled(sample))),
                    other => other.clone(),
                })
                .collect(),
        }
    }
}
impl TextureUnit {
    /// Freezes input weight, frame and UV animation while preserving sampling and texture source.
    pub fn sampled(&self, sample: &impl PropertySample) -> Self {
        let mut value = self.clone();
        value.contribution = self.contribution.sampled(sample);
        value.sampling.frame = self.sampling.frame.sampled(sample);
        value.coordinates.transform = self.coordinates.transform.sampled(sample);
        value
    }
}
impl M2MaterialStep<'_> {
    /// Freezes each shader's animated fields without changing its interpretation.
    pub fn sampled(&self, sample: &impl PropertySample) -> Self {
        let mut value = self.clone();
        let declaration = value.declaration_mut();
        declaration.color = declaration.color.sampled(sample);
        declaration.opacity = declaration.opacity.sampled(sample);
        for input in value.inputs_mut() {
            *input = input.sampled(sample);
        }
        value
    }
}
/// Borrowed color inputs compiled from a native surface. No image assets are required.
#[derive(Default)]
pub struct SurfaceColor<'a> {
    /// RGB override, taking precedence over RGBA RGB.
    pub color: Option<&'a Animated<[f32; 3]>>,
    /// RGBA surface multiplier.
    pub rgba: Option<&'a Animated<[f32; 4]>>,
    /// Explicit opacity multiplier.
    pub opacity: Option<&'a Animated<f32>>,
    /// Additional input weight.
    pub weight: Option<&'a Animated<f32>>,
    /// Ordered pass contribution weights.
    pub pass_weights: &'a [Animated<f32>],
    /// Authored untinted color.
    pub fallback: [f32; 4],
    /// Authored opacity before weight multiplication.
    pub unweighted_opacity: Option<f32>,
}
impl SurfaceColor<'_> {
    /// Samples linear RGBA and applies a geometry-owned tint.
    #[inline]
    pub fn sample(&self, sample: &impl PropertySample, tint: [f32; 4]) -> [f32; 4] {
        let rgb = self.color.map(|v| sample.animated(v));
        let rgba = self.rgba.map(|v| sample.animated(v));
        let rgb = rgb.or(rgba.map(|v| [v[0], v[1], v[2]])).unwrap_or([
            self.fallback[0],
            self.fallback[1],
            self.fallback[2],
        ]);
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
        [
            rgb[0] * tint[0],
            rgb[1] * tint[1],
            rgb[2] * tint[2],
            alpha * tint[3],
        ]
    }
}
