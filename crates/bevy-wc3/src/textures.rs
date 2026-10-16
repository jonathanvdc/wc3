//! Independent image interpretations selected by authored surface roles.
use bevy::prelude::{Handle, Image};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wc3::model::materials::{layered, programmed, world, SurfaceSet};
use wc3::model::{AddressMode, ColorSpace, Model, Sampler, Texture, TextureId};

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
            ColorSpace::Unspecified | ColorSpace::Unknown(_) => match role {
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

/// Identity of one texture's effective transfer function and addressing variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureImageKey {
    /// Source semantic texture identity, retaining sampler/producer declarations.
    pub id: TextureId,
    /// Effective transfer function for this use.
    pub color_space: TextureColorSpace,
    /// Effective addressing selected by this surface use.
    pub sampler: Sampler,
}

/// A semantic texture request with independently selected role and interpretation.
pub struct TextureRequest<'view, 'data> {
    /// Texture identity used by material selection and animation.
    pub id: TextureId,
    /// Complete resource, sampler and producer-specific declarations.
    pub texture: &'view Texture<'data>,
    /// Material purpose that selected an unspecified transfer function.
    pub role: TextureRole,
    /// Effective transfer function; established explicit declarations take precedence.
    pub color_space: TextureColorSpace,
    /// Effective addressing selected by this surface use.
    pub sampler: Sampler,
}
impl TextureRequest<'_, '_> {
    /// Cache identity for the resolved interpretation.
    pub fn key(&self) -> TextureImageKey {
        TextureImageKey {
            id: self.id,
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
    let mut add = |id: TextureId, role: TextureRole, sampler: Option<Sampler>| {
        if let Some(texture) = model.texture(id) {
            let request = TextureRequest {
                id,
                texture,
                role,
                color_space: TextureColorSpace::selected(texture.color_space, role),
                sampler: sampler.unwrap_or(texture.sampler),
            };
            if seen.insert(request.key()) {
                requests.push(request);
            }
        }
    };
    // Every image remains available for animated diffuse/replaceable selection.
    for (id, _) in model.textures() {
        add(id, TextureRole::Color, None);
    }
    for (_, material) in model.materials() {
        match &material.surface {
            SurfaceSet::Layered(surface) => {
                for layer in &surface.layers {
                    if !is_default_unit(surface, layer) {
                        continue;
                    }
                    for slot in &layer.textures {
                        let role = match slot.slot {
                            1 => TextureRole::Normal,
                            2 => TextureRole::Data,
                            _ => TextureRole::Color,
                        };
                        if let Some(id) = slot.binding.image.default {
                            add(id, role, None);
                        }
                        // Texture tracks may select any model-owned image. Their data
                        // interpretations must already exist before animation runs.
                        if slot.binding.image.property.is_some() {
                            for (id, _) in model.textures() {
                                add(id, role, None);
                            }
                        }
                    }
                }
            }
            SurfaceSet::World(surface) => {
                for slot in &surface.textures {
                    if let Some((id, texture)) = slot
                        .image
                        .and_then(|id| model.texture(id).map(|texture| (id, texture)))
                    {
                        add(
                            id,
                            TextureRole::Color,
                            Some(world_sampler(surface, texture.sampler)),
                        );
                    }
                }
            }
            SurfaceSet::Programmed(surface) => {
                if let programmed::Material::Standard(surface) = &**surface {
                    for layer in [
                        &surface.normals.base,
                        &surface.normals.primary.input,
                        &surface.normals.secondary.input,
                    ]
                    .into_iter()
                    .flatten()
                    {
                        if let Some(id) = layer.binding.image {
                            add(id, TextureRole::Normal, None);
                        }
                    }
                    for layer in [
                        &surface.normals.primary.mask,
                        &surface.normals.secondary.mask,
                        &surface.specular.input.gloss,
                        &surface.height_layer,
                        &surface.ambient_occlusion_layer,
                        &surface.environment_mask_layer,
                        &surface.opacity.primary,
                        &surface.opacity.secondary,
                    ]
                    .into_iter()
                    .flatten()
                    {
                        if let Some(id) = layer.binding.image {
                            add(id, TextureRole::Data, None);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    requests
}

/// Shared resolved images; source IDs, transfer functions, and samplers stay distinct.
#[derive(Clone, Default)]
pub(crate) struct TextureImages {
    variants: HashMap<TextureImageKey, Handle<Image>>,
    colors: Arc<HashMap<TextureId, Handle<Image>>>,
    declared: HashMap<TextureId, ColorSpace>,
    samplers: HashMap<TextureId, Sampler>,
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
        selected: Option<&HashSet<TextureId>>,
        mut resolver: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
    ) -> Self {
        let variants = texture_requests(model)
            .into_iter()
            .filter(|request| selected.is_none_or(|selected| selected.contains(&request.id)))
            .filter_map(|request| {
                let key = request.key();
                resolver(request).map(|image| (key, image))
            })
            .collect::<HashMap<_, _>>();
        let colors = model
            .textures()
            .filter_map(|(id, texture)| {
                let key = TextureImageKey {
                    id,
                    color_space: TextureColorSpace::selected(
                        texture.color_space,
                        TextureRole::Color,
                    ),
                    sampler: texture.sampler,
                };
                variants.get(&key).map(|image| (id, image.clone()))
            })
            .collect();
        Self {
            variants,
            colors: Arc::new(colors),
            samplers: model
                .textures()
                .map(|(id, texture)| (id, texture.sampler))
                .collect(),
            declared: model
                .textures()
                .map(|(id, texture)| (id, texture.color_space))
                .collect(),
        }
    }
    pub(crate) fn color_images(&self) -> &Arc<HashMap<TextureId, Handle<Image>>> {
        &self.colors
    }
    pub(crate) fn get(
        &self,
        id: TextureId,
        declared: ColorSpace,
        role: TextureRole,
    ) -> Option<&Handle<Image>> {
        self.get_with_sampler(id, declared, role, *self.samplers.get(&id)?)
    }
    pub(crate) fn get_with_sampler(
        &self,
        id: TextureId,
        declared: ColorSpace,
        role: TextureRole,
        sampler: Sampler,
    ) -> Option<&Handle<Image>> {
        self.variants.get(&TextureImageKey {
            id,
            color_space: TextureColorSpace::selected(declared, role),
            sampler,
        })
    }
    pub(crate) fn for_role(&self, id: TextureId, role: TextureRole) -> Option<&Handle<Image>> {
        self.get(id, *self.declared.get(&id)?, role)
    }
    #[cfg(test)]
    pub(crate) fn from_colors(
        model: &Model<'_>,
        images: HashMap<TextureId, Handle<Image>>,
    ) -> Self {
        Self::resolve(model, |request| images.get(&request.id).cloned())
    }
}

pub(crate) fn is_default_unit(
    surface: &layered::LayeredSurface<'_>,
    layer: &layered::Layer,
) -> bool {
    // The per-layer shader selector takes precedence in normalized newer files.
    match layer.shader {
        Some(1) => true,
        Some(_) => false,
        None => surface.shader == "Shader_HD_DefaultUnit",
    }
}

/// WMO addressing is surface authority; texture definitions remain unchanged.
pub(crate) fn world_sampler(surface: &world::Material<'_>, mut sampler: Sampler) -> Sampler {
    let controls = surface.state.rasterization.controls;
    if controls.clamp_u {
        sampler.u = AddressMode::Clamp;
    }
    if controls.clamp_v {
        sampler.v = AddressMode::Clamp;
    }
    sampler
}
#[cfg(test)]
#[path = "textures_tests.rs"]
mod tests;
