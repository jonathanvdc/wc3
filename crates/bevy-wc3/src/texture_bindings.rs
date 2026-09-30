//! Texture choices owned by each model instance.
use bevy::prelude::*;
use std::collections::HashMap;

use crate::asset::ResolvedModelTextures;
use crate::particle_render::ParticleInstances;

/// A texture use in the source model. Indices follow the MDX bitmap and PRE2 lists.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Wc3TextureSlot {
    Bitmap(usize),
    Particle2(usize),
}

#[derive(Component)]
pub(crate) struct ParticleTextureSlot(pub(crate) usize);

/// Per-instance texture choices. Attach this to a model root before spawning, or
/// mutate it later to change the images used by that instance.
#[derive(Component, Clone, Default)]
pub struct Wc3TextureBindings {
    replaceable: HashMap<u32, Handle<Image>>,
    slots: HashMap<Wc3TextureSlot, Handle<Image>>,
    pub(crate) defaults: ResolvedModelTextures,
}

pub(crate) fn update_particle_textures(
    roots: Query<&Wc3TextureBindings, Changed<Wc3TextureBindings>>,
    mut particles: Query<(
        &crate::particle2::Particle2State,
        &ParticleTextureSlot,
        &mut ParticleInstances,
    )>,
) {
    for (particle, slot, mut instances) in &mut particles {
        let Ok(bindings) = roots.get(particle.root) else {
            continue;
        };
        instances.texture = bindings.particle(slot.0);
    }
}

impl Wc3TextureBindings {
    pub fn set_replaceable(&mut self, id: u32, image: Handle<Image>) {
        self.replaceable.insert(id, image);
    }

    pub fn clear_replaceable(&mut self, id: u32) {
        self.replaceable.remove(&id);
    }

    pub fn set_slot(&mut self, slot: Wc3TextureSlot, image: Handle<Image>) {
        self.slots.insert(slot, image);
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::ResolvedTexture;
    use crate::particle2::Particle2State;
    use wc3::model::mdl::Read as _;
    use wc3::model::{Model, V1800};

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

    #[test]
    fn changing_particle_binding_updates_its_render_data() {
        let model =
            Model::<V1800>::decode_mdl(include_str!("../tests/fixtures/attachment_parent.mdl"))
                .unwrap();
        let emitter = model.particle_emitters2().remove(0);
        let mut app = App::new();
        app.add_systems(Update, update_particle_textures);
        let image = Assets::<Image>::default().add(Image::default());
        let root = app
            .world_mut()
            .spawn(
                Wc3TextureBindings::default().with_defaults(ResolvedModelTextures {
                    particles: vec![ResolvedTexture {
                        replaceable_id: 31,
                        ..default()
                    }],
                    ..default()
                }),
            )
            .id();
        let particle = app
            .world_mut()
            .spawn((
                Particle2State::new(root, root, emitter.clone()),
                ParticleTextureSlot(0),
                ParticleInstances {
                    particles: Vec::new(),
                    texture: None,
                    filter: emitter.filter_mode,
                    priority_plane: emitter.priority_plane,
                    sort_far: false,
                },
            ))
            .id();
        app.update();
        app.world_mut()
            .entity_mut(root)
            .get_mut::<Wc3TextureBindings>()
            .unwrap()
            .set_replaceable(31, image.clone());
        app.update();
        assert_eq!(
            app.world()
                .entity(particle)
                .get::<ParticleInstances>()
                .unwrap()
                .texture,
            Some(image)
        );
    }
}
