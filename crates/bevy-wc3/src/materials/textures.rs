//! Texture choices owned by each model instance.
use bevy::asset::AssetId;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::assets::loader::ResolvedModelTextures;

/// A texture use in the source model. Indices follow the MDX bitmap and PRE2 lists.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Wc3TextureSlot {
    /// A bitmap record index in the source texture list.
    Bitmap(usize),
    /// A PRE2 emitter record index, rather than its TextureID.
    Particle2(usize),
}

/// Per-instance texture choices. Attach this to a model root before spawning, or
/// mutate it later to change the images used by that instance.
///
/// Exact slot overrides take precedence over replaceable-ID bindings, followed
/// by resolved defaults. A PRE2 emitter with a zero replaceable ID falls back
/// to its TextureID bitmap binding. Other instances retain their own choices.
#[derive(Component, Clone, Default)]
pub struct Wc3TextureBindings {
    replaceable: HashMap<u32, Handle<Image>>,
    slots: HashMap<Wc3TextureSlot, Handle<Image>>,
    pub(crate) defaults: ResolvedModelTextures,
}

impl Wc3TextureBindings {
    /// Bind an image to all uses of this replaceable ID in the instance.
    pub fn set_replaceable(&mut self, id: u32, image: Handle<Image>) {
        self.replaceable.insert(id, image);
    }

    /// Remove an ID override, restoring resolved defaults where available.
    pub fn clear_replaceable(&mut self, id: u32) {
        self.replaceable.remove(&id);
    }

    /// Override one source slot, taking precedence over replaceable bindings.
    pub fn set_slot(&mut self, slot: Wc3TextureSlot, image: Handle<Image>) {
        self.slots.insert(slot, image);
    }

    /// Remove a slot override and resume ID/default resolution.
    pub fn clear_slot(&mut self, slot: Wc3TextureSlot) {
        self.slots.remove(&slot);
    }

    pub(crate) fn with_defaults(mut self, defaults: ResolvedModelTextures) -> Self {
        self.defaults = defaults;
        self
    }

    pub(crate) fn bitmap(&self, index: usize) -> Option<Handle<Image>> {
        let binding = self.defaults.bitmaps.get(index)?;
        self.slots
            .get(&Wc3TextureSlot::Bitmap(index))
            .cloned()
            .or_else(|| {
                (binding.replaceable_id != 0)
                    .then(|| self.replaceable.get(&binding.replaceable_id))
                    .flatten()
                    .cloned()
            })
            .or_else(|| binding.default.clone())
    }

    pub(crate) fn particle(&self, index: usize) -> Option<Handle<Image>> {
        let binding = self.defaults.particles.get(index)?;
        self.slots
            .get(&Wc3TextureSlot::Particle2(index))
            .cloned()
            .or_else(|| {
                (binding.replaceable_id != 0)
                    .then(|| self.replaceable.get(&binding.replaceable_id))
                    .flatten()
                    .cloned()
            })
            .or_else(|| binding.default.clone())
            .or_else(|| binding.bitmap_id.and_then(|bitmap| self.bitmap(bitmap)))
    }
}

#[derive(Default)]
pub(crate) struct LinearImages(HashMap<AssetId<Image>, Handle<Image>>);

impl LinearImages {
    pub(super) fn remove(&mut self, id: AssetId<Image>) {
        self.0.remove(&id);
    }

    pub(super) fn retain_loaded(&mut self, images: &Assets<Image>) {
        self.0.retain(|id, _| images.contains(*id));
    }

    pub(super) fn resolve(
        &mut self,
        source: Option<Handle<Image>>,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        let source = source?;
        let image = images.get(&source)?;
        let format = image.texture_descriptor.format.remove_srgb_suffix();
        if format == image.texture_descriptor.format {
            return Some(source);
        }
        if let Some(handle) = self.0.get(&source.id()) {
            return Some(handle.clone());
        }
        // A private variant preserves other materials' interpretation, sampler,
        // compression and mip chain, including consumer-supplied overrides.
        let mut linear = image.clone();
        linear.texture_descriptor.format = format;
        let handle = images.add(linear);
        self.0.insert(source.id(), handle.clone());
        Some(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loader::ResolvedTexture;
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    #[test]
    fn linear_variants_preserve_source_and_sampler_and_are_reused() {
        let mut images = Assets::<Image>::default();
        let source = images.add(Image::new(
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![128, 128, 255, 128],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ));
        let mut cache = LinearImages::default();
        let first = cache.resolve(Some(source.clone()), &mut images).unwrap();
        assert_ne!(first, source);
        assert_eq!(
            images.get(&source).unwrap().texture_descriptor.format,
            TextureFormat::Rgba8UnormSrgb
        );
        assert_eq!(
            images.get(&first).unwrap().texture_descriptor.format,
            TextureFormat::Rgba8Unorm
        );
        assert_eq!(
            images.get(&first).unwrap().data,
            images.get(&source).unwrap().data
        );
        assert_eq!(cache.resolve(Some(source), &mut images), Some(first));
    }

    #[test]
    fn slot_override_precedes_id_and_particle_texture_id_follows_bitmap() {
        let mut images = Assets::<Image>::default();
        let by_id = images.add(Image::default());
        let by_slot = images.add(Image::default());
        let bitmap = images.add(Image::default());
        let mut bindings = Wc3TextureBindings::default().with_defaults(ResolvedModelTextures {
            bitmaps: vec![ResolvedTexture {
                replaceable_id: 31,
                ..default()
            }],
            particles: vec![
                ResolvedTexture {
                    replaceable_id: 31,
                    ..default()
                },
                ResolvedTexture {
                    bitmap_id: Some(0),
                    ..default()
                },
            ],
        });
        bindings.set_replaceable(31, by_id.clone());
        assert_eq!(bindings.particle(0), Some(by_id.clone()));
        assert_eq!(bindings.particle(1), Some(by_id.clone()));
        bindings.set_slot(Wc3TextureSlot::Bitmap(0), bitmap.clone());
        assert_eq!(bindings.particle(1), Some(bitmap));
        bindings.set_slot(Wc3TextureSlot::Particle2(0), by_slot.clone());
        assert_eq!(bindings.particle(0), Some(by_slot));
        bindings.clear_slot(Wc3TextureSlot::Particle2(0));
        assert_eq!(bindings.particle(0), Some(by_id));
    }
}
