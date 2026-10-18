//! Instance-local image sources and immutable request interpretation variants.
use crate::child_coordinates::ChildCoordinates;
use crate::effects::model_particles::ModelParticleChild;
use crate::textures::{ImageKey, TextureRequestKey, TextureRole};
use crate::{ModelAsset, ModelInstance, ModelSource, PreparedModel};
use bevy::asset::{AssetEvent, AssetId};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Weak};
use wc3::model::{AddressMode, ImageResource, ImageResourceId};

/// Image source overrides for an asset-backed model instance.
///
/// Request overrides take precedence over resource overrides, then replaceable
/// categories. Request and resource identities belong to their source model.
/// Categories inherit through authored attachment and model-particle ownership;
/// a child's category wins over its owner's category. Application children do
/// not inherit replacements. Effective image variants retain each request's
/// sampler and color interpretation without modifying the supplied source image.
///
/// Changing or removing this component rebinds the renderer-owned hierarchy.
/// Root playback and pose overrides survive; attachment playback, physics and
/// transient particle/trail populations restart. Use stable bindings when those
/// independently clocked states must persist.
#[derive(Component, Clone, Default, Debug, PartialEq, Eq)]
pub struct ImageReplacements {
    /// Exact logical image requests in this model, including animated selections.
    pub requests: HashMap<TextureRequestKey, Handle<Image>>,
    /// All uses of one source-model image declaration.
    pub resources: HashMap<ImageResourceId, Handle<Image>>,
    /// Authored replaceable category IDs, shared with authored child models.
    pub categories: HashMap<u32, Handle<Image>>,
}
impl ImageReplacements {
    /// Replace all uses of one native replaceable category.
    pub fn replaceable(id: u32, image: Handle<Image>) -> Self {
        Self {
            categories: HashMap::from([(id, image)]),
            ..Default::default()
        }
    }
    fn source(&self, binding: &ImageBinding) -> Option<&Handle<Image>> {
        self.requests
            .get(&binding.key)
            .or_else(|| self.resources.get(&binding.key.resource))
            .or_else(|| match &binding.resource {
                ImageResource::Replaceable { id, .. } => self.categories.get(id),
                _ => None,
            })
    }
    fn is_empty(&self) -> bool {
        self.requests.is_empty() && self.resources.is_empty() && self.categories.is_empty()
    }
}

#[derive(Clone)]
pub(crate) struct ImageBinding {
    pub key: TextureRequestKey,
    pub resource: ImageResource<'static>,
    pub role: TextureRole,
    pub symbol: Handle<Image>,
    pub default: Option<Handle<Image>>,
}

#[derive(Component, Clone)]
pub(crate) struct BoundImages {
    replacements: ImageReplacements,
    generation: u64,
    _prepared: Arc<PreparedModel>,
}

#[derive(Resource, Default)]
pub(crate) struct ReplacementImages {
    generation: u64,
    variants: HashMap<ImageKey<AssetId<Image>>, Handle<Image>>,
    models: Vec<(AssetId<ModelAsset>, ImageReplacements, Weak<PreparedModel>)>,
}

pub(crate) type ImageOwners<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static ChildOf>,
        Option<&'static ModelParticleChild>,
        Option<&'static ChildCoordinates>,
        Option<&'static ModelSource>,
        Option<&'static ModelInstance>,
        Option<&'static ImageReplacements>,
    ),
>;

/// Compute only renderer-authored ownership inheritance; model-local identities
/// never escape the model on which the component was supplied.
pub(crate) fn effective_replacements(
    root: Entity,
    owners: &ImageOwners<'_, '_>,
) -> ImageReplacements {
    let Ok((parent, particle, placement, _, _, local)) = owners.get(root) else {
        return ImageReplacements::default();
    };
    let mut result = local.cloned().unwrap_or_default();
    if particle.is_none() && placement.is_none() {
        return result;
    }
    let mut current = particle
        .map(ModelParticleChild::parent_model)
        .or_else(|| parent.map(ChildOf::parent));
    let mut visited = HashSet::new();
    while let Some(entity) = current {
        if !visited.insert(entity) {
            break;
        }
        let Ok((parent, particle, placement, source, request, replacements)) = owners.get(entity)
        else {
            break;
        };
        if let Some(replacements) = replacements {
            for (&id, image) in &replacements.categories {
                result.categories.entry(id).or_insert_with(|| image.clone());
            }
        }
        if (source.is_some() || request.is_some()) && particle.is_none() && placement.is_none() {
            break;
        }
        current = particle
            .map(ModelParticleChild::parent_model)
            .or_else(|| parent.map(ChildOf::parent));
    }
    result
}

impl ReplacementImages {
    fn prune(&mut self) {
        self.models.retain(|(_, _, model)| model.strong_count() > 0);
        let sources: HashSet<_> = self
            .models
            .iter()
            .flat_map(|(_, bindings, _)| {
                bindings
                    .requests
                    .values()
                    .chain(bindings.resources.values())
                    .chain(bindings.categories.values())
                    .map(Handle::id)
            })
            .collect();
        self.variants.retain(|key, _| sources.contains(&key.source));
    }
    pub(crate) fn invalidate(&mut self, id: AssetId<ModelAsset>) {
        self.models.retain(|(source, _, _)| *source != id);
    }
    fn bind(
        &mut self,
        original: &PreparedModel,
        replacements: &ImageReplacements,
        images: &mut Assets<Image>,
    ) -> Option<PreparedModel> {
        let mut prepared = if replacements.is_empty() {
            original.clone()
        } else {
            original
                .image_template
                .as_deref()
                .unwrap_or(original)
                .clone()
        };
        if !replacements.is_empty() {
            let mut resolved = HashMap::new();
            for binding in prepared.image_bindings.iter() {
                let image = if let Some(source) = replacements.source(binding) {
                    let key = ImageKey {
                        source: source.id(),
                        color_space: binding.key.color_space,
                        sampler: binding.key.sampler,
                    };
                    let handle = if let Some(handle) = self.variants.get(&key) {
                        handle.clone()
                    } else {
                        let mut image = images.get(source)?.clone();
                        image.texture_descriptor.format = if key.color_space.is_srgb() {
                            image.texture_descriptor.format.add_srgb_suffix()
                        } else {
                            image.texture_descriptor.format.remove_srgb_suffix()
                        };
                        let mut sampler = match &image.sampler {
                            ImageSampler::Descriptor(sampler) => sampler.clone(),
                            _ => ImageSamplerDescriptor::linear(),
                        };
                        let address = |mode| match mode {
                            AddressMode::Repeat => ImageAddressMode::Repeat,
                            AddressMode::Mirror => ImageAddressMode::MirrorRepeat,
                            _ => ImageAddressMode::ClampToEdge,
                        };
                        sampler.address_mode_u = address(key.sampler.u);
                        sampler.address_mode_v = address(key.sampler.v);
                        image.sampler = ImageSampler::Descriptor(sampler);
                        let handle = images.add(image);
                        self.variants.insert(key, handle.clone());
                        handle
                    };
                    Some(handle)
                } else {
                    binding.default.clone()
                };
                resolved.insert(binding.symbol.id(), image);
            }
            prepared.map_images(&mut |image| {
                resolved
                    .get(&image.id())
                    .cloned()
                    .unwrap_or_else(|| Some(image.clone()))
            });
        }
        let inherited = ImageReplacements {
            categories: replacements.categories.clone(),
            ..Default::default()
        };
        let mut mounts = original.mounts.to_vec();
        for mount in &mut mounts {
            mount.model = self.bind(&mount.model, &inherited, images)?;
        }
        prepared.mounts = mounts.into();
        Some(prepared)
    }
    pub(crate) fn prepare(
        &mut self,
        model_id: AssetId<ModelAsset>,
        original: &PreparedModel,
        replacements: ImageReplacements,
        images: &mut Assets<Image>,
    ) -> Option<(PreparedModel, BoundImages)> {
        self.models.retain(|(_, _, model)| model.strong_count() > 0);
        if let Some(prepared) = self.models.iter().find_map(|(id, bindings, model)| {
            (*id == model_id && *bindings == replacements)
                .then(|| model.upgrade())
                .flatten()
        }) {
            return Some((
                (*prepared).clone(),
                BoundImages {
                    replacements,
                    generation: self.generation,
                    _prepared: prepared,
                },
            ));
        }
        let prepared = self.bind(original, &replacements, images)?;
        let prepared = Arc::new(prepared);
        self.models
            .push((model_id, replacements.clone(), Arc::downgrade(&prepared)));
        Some((
            (*prepared).clone(),
            BoundImages {
                replacements,
                generation: self.generation,
                _prepared: prepared,
            },
        ))
    }
}

pub(crate) fn refresh_image_replacements(
    mut commands: Commands,
    owners: ImageOwners<'_, '_>,
    sources: Query<(Entity, &ModelSource, Option<&BoundImages>), Without<ModelInstance>>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut cache: ResMut<ReplacementImages>,
) {
    cache.prune();
    let changed = events.read().fold(false, |changed, event| changed | matches!(event, AssetEvent::Modified { id } | AssetEvent::Removed { id } if cache.variants.keys().any(|key| key.source == *id)));
    if changed {
        cache.variants.clear();
        cache.models.clear();
        cache.generation = cache.generation.wrapping_add(1);
    }
    for (root, source, bound) in &sources {
        let replacements = effective_replacements(root, &owners);
        if bound.is_none_or(|bound| {
            bound.replacements != replacements
                || (!replacements.is_empty() && bound.generation != cache.generation)
        }) {
            commands
                .entity(root)
                .insert(ModelInstance(source.0.clone()));
        }
    }
}

#[cfg(test)]
#[path = "image_replacements_tests.rs"]
mod tests;
