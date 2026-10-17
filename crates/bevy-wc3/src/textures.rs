//! Independent image interpretations selected by authored surface roles.
use bevy::prelude::{Handle, Image};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wc3::model::materials::program::MaterialStep;
use wc3::model::materials::{m3, mdlx};
use wc3::model::{
    AddressMode, ColorSpace, FeatureId, ImageResource, ImageResourceId, MaterialId, Model,
    NodeFeature, ParticleDetails, ParticleEmitter, Sampler, Texture, TextureControls,
};

/// Purpose of one sampled image, independent of its resource identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureRole {
    /// Diffuse, emission, tint, or other authored color response.
    Color,
    /// Tangent-space normal components.
    Normal,
    /// Masks, roughness, occlusion, gloss, or displacement values.
    Data,
}

/// Effective transfer function for the sampled image.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureColorSpace {
    /// Color samples decode the sRGB transfer function.
    Srgb,
    /// Numeric data or explicitly linear colors are sampled unchanged.
    Linear,
}
impl TextureColorSpace {
    pub(crate) fn selected(declared: ColorSpace, role: TextureRole) -> Self {
        match declared {
            ColorSpace::Srgb => Self::Srgb,
            ColorSpace::Linear => Self::Linear,
            ColorSpace::Unspecified => match role {
                TextureRole::Color => Self::Srgb,
                TextureRole::Normal | TextureRole::Data => Self::Linear,
            },
        }
    }
    /// Whether the GPU image format should decode sRGB samples.
    pub fn is_srgb(self) -> bool {
        self == Self::Srgb
    }
}

/// Structural address of an image input in its source model.
/// Material slots follow the native input order (MDLX uses shader slot numbers).
/// M3 Standard uses diffuse, decal, specular, gloss, primary/secondary emission,
/// primary/secondary opacity, base normal, primary normal/mask, secondary
/// normal/mask, light map, ambient occlusion, height, environment and environment mask.
/// Particle slots follow the emitter's image order; atlas emitters use slot zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureInput {
    /// One material step's image input.
    Material {
        /// Owning material.
        material: MaterialId,
        /// Zero-based step in the material program.
        step: u32,
        /// Image slot within that step.
        slot: u32,
    },
    /// One sprite emitter's image input.
    Particle {
        /// Owning sprite emitter.
        feature: FeatureId,
        /// Image slot within the emitter.
        slot: u32,
    },
}
impl TextureInput {
    fn with_slot(self, slot: u32) -> Self {
        match self {
            Self::Material { material, step, .. } => Self::Material {
                material,
                step,
                slot,
            },
            Self::Particle { feature, .. } => Self::Particle { feature, slot },
        }
    }
}

/// Physical image identity after source resolution and replacement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ImageKey<S> {
    pub(crate) source: S,
    pub(crate) color_space: TextureColorSpace,
    pub(crate) sampler: Sampler,
}

/// Logical lookup for one input's selected image interpretation.
/// This identifies a renderer request, rather than a physical GPU image.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureRequestKey {
    /// Input occurrence, or an unscoped image selection.
    pub input: Option<TextureInput>,
    /// Selected image dependency.
    pub resource: ImageResourceId,
    /// Authored transfer declaration used by animated selections.
    pub declared: ColorSpace,
    /// Native controls available to source resolution.
    pub controls: Option<TextureControls>,
    /// Effective transfer function for this use.
    pub color_space: TextureColorSpace,
    /// Effective addressing selected by this surface use.
    pub sampler: Sampler,
}

/// A semantic texture request with independently selected role and interpretation.
pub struct TextureRequest<'view, 'data> {
    /// Structural input address; absent for unscoped animated image availability.
    pub input: Option<TextureInput>,
    /// Image handle, sampler, and producer-specific texture declarations.
    pub texture: Texture,
    /// Image declaration resolved from the owning model.
    pub resource: &'view ImageResource<'data>,
    /// Material purpose that selected an unspecified transfer function.
    pub role: TextureRole,
    /// Effective transfer function; established explicit declarations take precedence.
    pub color_space: TextureColorSpace,
    /// Effective addressing selected by this surface use.
    pub sampler: Sampler,
}
impl TextureRequest<'_, '_> {
    /// Cache identity for the resolved interpretation.
    pub fn key(&self) -> TextureRequestKey {
        TextureRequestKey {
            input: self.input,
            resource: self.texture.resource,
            declared: self.texture.color_space,
            controls: self.texture.controls,
            color_space: self.color_space,
            sampler: self.sampler,
        }
    }
}

/// Collect image uses needed by supported surface roles and default color selection.
/// A texture used as both color and data receives independent requests.
/// M3 roles share GPU samplers for equal authored sampler values. Resolvers must
/// therefore use equal filtering, mipmap and anisotropy settings for those requests.
pub fn texture_requests<'view, 'data>(
    model: &'view Model<'data>,
) -> Vec<TextureRequest<'view, 'data>> {
    let mut requests = Vec::new();
    let mut seen = HashSet::new();
    let mut add = |input, id: Texture, role: TextureRole, sampler: Option<Sampler>| {
        let mut texture = id;
        texture.sampler = sampler.unwrap_or(texture.sampler);
        let request = TextureRequest {
            input,
            texture,
            resource: model
                .image_resource(texture.resource)
                .expect("validated image resource"),
            role,
            color_space: TextureColorSpace::selected(texture.color_space, role),
            sampler: texture.sampler,
        };
        if seen.insert(request.key()) {
            requests.push(request);
        }
    };
    for image in model.sampled_images() {
        add(None, image, TextureRole::Color, None);
    }
    for (material_id, material) in model.materials() {
        for (step, pass) in material.surface.steps.iter().enumerate() {
            let address = |slot| {
                Some(TextureInput::Material {
                    material: material_id,
                    step: step as u32,
                    slot,
                })
            };
            match pass {
                MaterialStep::Mdlx(layer) => {
                    let base = match layer.as_ref() {
                        mdlx::MaterialStep::Classic(layer) => &layer.inputs.binding.image,
                        mdlx::MaterialStep::ReforgedDefault(layer) => &layer.inputs.binding.image,
                        mdlx::MaterialStep::Unresolved(layer) => &layer.layer.binding.image,
                    };
                    let mut slots = vec![(0, base)];
                    if let mdlx::MaterialStep::ReforgedDefault(layer) = layer.as_ref() {
                        for slot in &layer.textures {
                            slots.retain(|(index, _)| *index != slot.slot);
                            slots.push((slot.slot, &slot.binding.image));
                        }
                    }
                    for (slot, binding) in slots {
                        let role = match slot {
                            1 => TextureRole::Normal,
                            2 => TextureRole::Data,
                            _ => TextureRole::Color,
                        };
                        if let Some(image) = binding.default {
                            add(address(slot), image, role, None);
                        }
                        if binding.binding.is_some() {
                            for image in model.sampled_images() {
                                add(address(slot), image, role, None);
                            }
                        }
                    }
                }
                MaterialStep::M2(surface) => {
                    for (slot, input) in surface.inputs().iter().enumerate() {
                        if let Some(image) = input.image.texture {
                            add(address(slot as u32), image, TextureRole::Color, None);
                        }
                    }
                }
                MaterialStep::Wmo(surface) => {
                    for (slot, input) in surface.inputs().iter().enumerate() {
                        if let Some(image) = input.image {
                            add(
                                address(slot as u32),
                                Texture::new(image),
                                TextureRole::Color,
                                Some(input.sampling),
                            );
                        }
                    }
                }
                MaterialStep::M3(m3::MaterialStep::Standard(surface)) => {
                    for (slot, layer) in m3_layers(surface).into_iter().enumerate() {
                        let Some(layer) = layer else { continue };
                        if let Some(image) = layer.binding.image {
                            let role = match slot {
                                0 | 1 | 2 | 4 | 5 | 13 | 16 => TextureRole::Color,
                                8 | 9 | 11 => TextureRole::Normal,
                                _ => TextureRole::Data,
                            };
                            add(
                                address(slot as u32),
                                Texture::new(image),
                                role,
                                Some(m3_sampler(layer)),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }
    for (feature, value) in model.features() {
        if let NodeFeature::Particle(ParticleEmitter::Simulated(particle)) = &value.payload {
            let images = match &particle.details {
                ParticleDetails::AgeDriven(details) => details.rendering.textures.to_vec(),
                ParticleDetails::Atlas(details) => details.rendering.texture.into_iter().collect(),
                _ => Vec::new(),
            };
            for (slot, image) in images.into_iter().enumerate() {
                add(
                    Some(TextureInput::Particle {
                        feature,
                        slot: slot as u32,
                    }),
                    image,
                    TextureRole::Color,
                    None,
                );
            }
        }
    }
    let unscoped = requests
        .iter()
        .filter(|request| request.input.is_some())
        .map(|request| TextureRequest {
            input: None,
            texture: request.texture,
            resource: request.resource,
            role: request.role,
            color_space: request.color_space,
            sampler: request.sampler,
        })
        .collect::<Vec<_>>();
    for request in unscoped {
        if seen.insert(request.key()) {
            requests.push(request);
        }
    }
    requests
}

/// Stable input ordering shared by requests and the Standard material interpreter.
pub(crate) fn m3_layers(surface: &m3::StandardMaterialStep) -> [Option<&m3::TextureLayer>; 18] {
    [
        surface.diffuse.as_ref(),
        surface.decal_layer.as_ref(),
        surface.specular.input.color.as_ref(),
        surface.specular.input.gloss.as_ref(),
        surface.emissive.input.primary.input.as_ref(),
        surface.emissive.input.secondary.input.as_ref(),
        surface.opacity.primary.as_ref(),
        surface.opacity.secondary.as_ref(),
        surface.normals.base.as_ref(),
        surface.normals.primary.input.as_ref(),
        surface.normals.primary.mask.as_ref(),
        surface.normals.secondary.input.as_ref(),
        surface.normals.secondary.mask.as_ref(),
        surface.light_map_layer.as_ref(),
        surface.ambient_occlusion_layer.as_ref(),
        surface.height_layer.as_ref(),
        surface.environment_layer.as_ref(),
        surface.environment_mask_layer.as_ref(),
    ]
}

/// M3's typed sampling programme owns its address selectors.
pub(crate) fn m3_sampler(layer: &m3::TextureLayer) -> Sampler {
    let flags = layer.binding.sampling.flags;
    Sampler {
        u: if flags.uv_wrap_x() {
            AddressMode::Repeat
        } else {
            AddressMode::Clamp
        },
        v: if flags.uv_wrap_y() {
            AddressMode::Repeat
        } else {
            AddressMode::Clamp
        },
    }
}

/// Shared resolved images; source IDs, transfer functions, and samplers stay distinct.
#[derive(Clone, Default)]
pub(crate) struct TextureImages {
    variants: Arc<HashMap<TextureRequestKey, Handle<Image>>>,
    input: Option<TextureInput>,
}
impl TextureImages {
    pub(crate) fn resolve(
        model: &Model<'_>,
        resolver: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
    ) -> Self {
        Self::resolve_selected(model, None, resolver)
    }
    pub(crate) fn resolve_selected(
        model: &Model<'_>,
        selected: Option<&HashSet<Texture>>,
        mut resolver: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
    ) -> Self {
        let variants = texture_requests(model)
            .into_iter()
            .filter(|request| {
                selected.is_none_or(|selected| {
                    selected
                        .iter()
                        .any(|image| image.resource == request.texture.resource)
                })
            })
            .filter_map(|request| {
                let key = request.key();
                resolver(request).map(|image| (key, image))
            })
            .collect();
        Self {
            variants: Arc::new(variants),
            input: None,
        }
    }
    pub(crate) fn material(&self, material: MaterialId, step: u32) -> Self {
        Self {
            variants: self.variants.clone(),
            input: Some(TextureInput::Material {
                material,
                step,
                slot: 0,
            }),
        }
    }
    pub(crate) fn particle(&self, feature: FeatureId, slot: u32) -> Self {
        Self {
            variants: self.variants.clone(),
            input: Some(TextureInput::Particle { feature, slot }),
        }
    }
    pub(crate) fn slot(&self, slot: u32) -> Self {
        Self {
            variants: self.variants.clone(),
            input: self.input.map(|input| input.with_slot(slot)),
        }
    }
    pub(crate) fn color_images(&self) -> Arc<HashMap<Texture, Handle<Image>>> {
        Arc::new(
            self.variants
                .iter()
                .filter(|(key, _)| key.input == self.input)
                .filter_map(|(key, image)| {
                    // Animation selects authored Texture values. Retain both unspecified
                    // and explicit color declarations when they select this interpretation.
                    let texture = Texture {
                        resource: key.resource,
                        controls: key.controls,
                        color_space: key.declared,
                        sampler: key.sampler,
                    };
                    (key.color_space
                        == TextureColorSpace::selected(key.declared, TextureRole::Color))
                    .then_some((texture, image.clone()))
                })
                .collect(),
        )
    }
    pub(crate) fn get(
        &self,
        texture: Texture,
        declared: ColorSpace,
        role: TextureRole,
    ) -> Option<&Handle<Image>> {
        self.get_with_sampler(texture, declared, role, texture.sampler)
    }
    pub(crate) fn get_with_sampler(
        &self,
        texture: Texture,
        declared: ColorSpace,
        role: TextureRole,
        sampler: Sampler,
    ) -> Option<&Handle<Image>> {
        let key = TextureRequestKey {
            input: self.input,
            resource: texture.resource,
            declared: texture.color_space,
            controls: texture.controls,
            color_space: TextureColorSpace::selected(declared, role),
            sampler,
        };
        self.lookup(key)
    }
    fn lookup(&self, key: TextureRequestKey) -> Option<&Handle<Image>> {
        self.variants.get(&key)
    }

    pub(crate) fn for_role(&self, texture: Texture, role: TextureRole) -> Option<&Handle<Image>> {
        self.get(texture, texture.color_space, role)
    }
    pub(crate) fn for_slot(
        &self,
        slot: u32,
        texture: Texture,
        role: TextureRole,
    ) -> Option<&Handle<Image>> {
        self.lookup(TextureRequestKey {
            input: self.input.map(|input| input.with_slot(slot)),
            resource: texture.resource,
            declared: texture.color_space,
            controls: texture.controls,
            color_space: TextureColorSpace::selected(texture.color_space, role),
            sampler: texture.sampler,
        })
    }
    #[cfg(test)]
    pub(crate) fn from_colors(model: &Model<'_>, images: HashMap<Texture, Handle<Image>>) -> Self {
        Self::resolve(model, |request| {
            images.get(&request.texture).cloned().or_else(|| {
                images
                    .iter()
                    .find(|(image, _)| image.resource == request.texture.resource)
                    .map(|(_, handle)| handle.clone())
            })
        })
    }
}

#[cfg(test)]
#[path = "textures_tests.rs"]
mod tests;
