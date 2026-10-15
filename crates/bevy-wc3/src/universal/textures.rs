//! Independent image interpretations selected by authored surface roles.
use bevy::prelude::{Handle, Image};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wc3::model::materials::{programmed, SurfaceSet};
use wc3::model::{ColorSpace, Model, Texture, TextureId};

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

/// Identity of one texture's effective transfer-function variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureImageKey {
    /// Source semantic texture identity, retaining sampler/producer declarations.
    pub id: TextureId,
    /// Effective transfer function for this use.
    pub color_space: TextureColorSpace,
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
}
impl TextureRequest<'_, '_> {
    /// Cache identity for the resolved interpretation.
    pub fn key(&self) -> TextureImageKey {
        TextureImageKey {
            id: self.id,
            color_space: self.color_space,
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
    let mut add = |id: TextureId, role: TextureRole| {
        if let Some(texture) = model.texture(id) {
            let request = TextureRequest {
                id,
                texture,
                role,
                color_space: TextureColorSpace::selected(texture.color_space, role),
            };
            if seen.insert(request.key()) {
                requests.push(request);
            }
        }
    };
    // Every image remains available for animated diffuse/replaceable selection.
    for (id, _) in model.textures() {
        add(id, TextureRole::Color);
    }
    for (_, material) in model.materials() {
        match &material.surface {
            SurfaceSet::Layered(surface) => {
                for layer in &surface.layers {
                    for slot in &layer.textures {
                        let role = match slot.slot {
                            1 => TextureRole::Normal,
                            2 => TextureRole::Data,
                            _ => TextureRole::Color,
                        };
                        if let Some(id) = slot.binding.image.default {
                            add(id, role);
                        }
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
                            add(id, TextureRole::Normal);
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
                            add(id, TextureRole::Data);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    requests
}

/// Shared resolved images; source IDs and transfer functions remain distinct.
#[derive(Clone, Default)]
pub(crate) struct TextureImages {
    variants: HashMap<TextureImageKey, Handle<Image>>,
    colors: Arc<HashMap<TextureId, Handle<Image>>>,
}
impl TextureImages {
    pub(crate) fn resolve(
        model: &Model<'_>,
        mut resolver: impl FnMut(TextureRequest<'_, '_>) -> Option<Handle<Image>>,
    ) -> Self {
        let variants = texture_requests(model)
            .into_iter()
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
                };
                variants.get(&key).map(|image| (id, image.clone()))
            })
            .collect();
        Self {
            variants,
            colors: Arc::new(colors),
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
        self.variants.get(&TextureImageKey {
            id,
            color_space: TextureColorSpace::selected(declared, role),
        })
    }
    #[cfg(test)]
    pub(crate) fn from_colors(
        model: &Model<'_>,
        images: HashMap<TextureId, Handle<Image>>,
    ) -> Self {
        Self::resolve(model, |request| images.get(&request.id).cloned())
    }
}
