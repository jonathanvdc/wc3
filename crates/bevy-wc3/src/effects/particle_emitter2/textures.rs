use super::{Particle2State, ParticleInstances, ParticleTextureSlot};
use crate::materials::textures::Wc3TextureBindings;
use bevy::prelude::*;

pub(crate) fn update_particle_textures(
    roots: Query<&Wc3TextureBindings, Changed<Wc3TextureBindings>>,
    mut particles: Query<(
        &Particle2State,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::loader::{ResolvedModelTextures, ResolvedTexture};
    use wc3::model::mdl::Read as _;
    use wc3::model::{Model, V1800};
    #[test]
    fn changing_particle_binding_updates_its_render_data() {
        let model = Model::<V1800>::decode_mdl(include_str!(
            "../../../tests/fixtures/attachment_parent.mdl"
        ))
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
                    records: Default::default(),
                    live_indices: Default::default(),
                    uniform: Default::default(),
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
