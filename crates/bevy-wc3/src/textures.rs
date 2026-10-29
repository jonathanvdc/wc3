//! Independent image interpretations selected by authored surface roles.
#[cfg(test)]
#[path = "textures_tests.rs"]
mod tests;

use bevy::prelude::{Handle, Image};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wc3::model::materials::mdlx;
use wc3::model::materials::program::MaterialStep;
use wc3::model::{
    ColorSpace, FeatureId, ImageResource, ImageResourceId, MaterialId, Model, NodeFeature,
    ParticleEmitter, RibbonEmitter, Sampler, Texture,
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
    /// One ribbon emitter's ordered image input.
    Ribbon {
        /// Owning ribbon emitter.
        feature: FeatureId,
        /// Image slot within the emitter.
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
            Self::Ribbon { feature, .. } => Self::Ribbon { feature, slot },
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
    /// Effective transfer function for this use.
    pub color_space: TextureColorSpace,
    /// Effective addressing selected by this surface use.
    pub sampler: Sampler,
}

/// A semantic texture request with independently selected role and interpretation.
pub struct TextureRequest<'view, 'data> {
    /// Structural input address; absent for unscoped animated image availability.
    pub input: Option<TextureInput>,
    /// Image declaration handle, sampler, and color interpretation.
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
            color_space: self.color_space,
            sampler: self.sampler,
        }
    }
}

/// Collect image uses needed by supported surface roles and default color selection.
/// A texture used as both color and data receives independent requests.
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
                    let base = &layer.shader.inputs().binding.image;
                    let mut slots = vec![(0, base)];
                    let crystal = matches!(layer.shader, mdlx::Shader::ReforgedCrystal(_));
                    if let mdlx::Shader::ReforgedDefault(layer)
                    | mdlx::Shader::ReforgedCrystal(layer) = &layer.shader
                    {
                        for slot in &layer.textures {
                            if crystal && slot.slot == 5 {
                                continue;
                            }
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
                        if let Some(image) = model.values()[*binding] {
                            add(address(slot), image, role, None);
                        }
                        if model.is_animated(*binding) {
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
            }
        }
    }
    for (feature, value) in model.features() {
        if let NodeFeature::Ribbon(RibbonEmitter::M2(ribbon)) = &value.payload {
            for (slot, image) in ribbon.rendering.textures.iter().enumerate() {
                add(
                    Some(TextureInput::Ribbon {
                        feature,
                        slot: slot as u32,
                    }),
                    *image,
                    TextureRole::Color,
                    None,
                );
            }
        }
        if let NodeFeature::Particle(particle) = &value.payload {
            let images = match particle {
                ParticleEmitter::M2(value) => value.details.rendering.textures.to_vec(),
                ParticleEmitter::MdlxEmitter2(value) => {
                    value.details.rendering.texture.into_iter().collect()
                }
                _ => Vec::new(),
            };
            let role = match particle {
                ParticleEmitter::M2(value)
                    if value.details.flags.refraction() && !value.details.flags.multi_texture() =>
                {
                    TextureRole::Data
                }
                _ => TextureRole::Color,
            };
            for (slot, image) in images.into_iter().enumerate() {
                add(
                    Some(TextureInput::Particle {
                        feature,
                        slot: slot as u32,
                    }),
                    image,
                    role,
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

type ImageVariants = HashMap<(Texture, TextureColorSpace), Handle<Image>>;

/// Shared resolved images indexed by input; interpretations remain distinct within each scope.
#[derive(Clone, Default)]
pub(crate) struct TextureImages {
    variants: Arc<HashMap<Option<TextureInput>, ImageVariants>>,
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
        let selected = selected.map(|textures| {
            textures
                .iter()
                .map(|texture| texture.resource)
                .collect::<HashSet<_>>()
        });
        let mut variants: HashMap<_, ImageVariants> = HashMap::new();
        for request in texture_requests(model) {
            if selected
                .as_ref()
                .is_some_and(|resources| !resources.contains(&request.texture.resource))
            {
                continue;
            }
            let input = request.input;
            let texture = Texture {
                sampler: request.sampler,
                ..request.texture
            };
            let color_space = request.color_space;
            if let Some(image) = resolver(request) {
                variants
                    .entry(input)
                    .or_default()
                    .insert((texture, color_space), image);
            }
        }
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
    pub(crate) fn ribbon(&self, feature: FeatureId, slot: u32) -> Self {
        Self {
            variants: self.variants.clone(),
            input: Some(TextureInput::Ribbon { feature, slot }),
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
                .get(&self.input)
                .into_iter()
                .flat_map(|variants| variants.iter())
                .filter_map(|((texture, color_space), image)| {
                    // Animation selects authored Texture values. Retain both unspecified
                    // and explicit color declarations when they select this interpretation.
                    (*color_space
                        == TextureColorSpace::selected(texture.color_space, TextureRole::Color))
                    .then_some((*texture, image.clone()))
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
        self.lookup(
            self.input,
            Texture { sampler, ..texture },
            TextureColorSpace::selected(declared, role),
        )
    }
    pub(crate) fn map_images(
        &mut self,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        self.variants = Arc::new(
            self.variants
                .iter()
                .map(|(input, variants)| {
                    let variants = variants
                        .iter()
                        .filter_map(|(key, image)| map(image).map(|image| (*key, image)))
                        .collect();
                    (*input, variants)
                })
                .collect(),
        );
    }
    fn lookup(
        &self,
        input: Option<TextureInput>,
        texture: Texture,
        color_space: TextureColorSpace,
    ) -> Option<&Handle<Image>> {
        self.variants.get(&input)?.get(&(texture, color_space))
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
        self.lookup(
            self.input.map(|input| input.with_slot(slot)),
            texture,
            TextureColorSpace::selected(texture.color_space, role),
        )
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
