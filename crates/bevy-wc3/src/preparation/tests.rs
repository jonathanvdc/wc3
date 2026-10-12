use super::*;
use crate::animation::Wc3Animation;
use crate::instance::spawn::spawn_prepared_model;
use crate::materials::textures::Wc3TextureBindings;
use bevy::ecs::world::CommandQueue;

#[test]
fn prepared_features_release_the_container_and_spawn_after_source_drop() {
    for text in [
        include_str!("../../tests/fixtures/attachment_parent.mdl"),
        include_str!("../../tests/fixtures/cameras.mdl"),
        include_str!("../../tests/fixtures/events.mdl"),
        include_str!("../../tests/fixtures/light_capture.mdl"),
        include_str!("../../tests/fixtures/ribbon_capture.mdl"),
        include_str!("../../tests/fixtures/particle_capture.mdl"),
        include_str!("../../tests/fixtures/prem_capture.mdl"),
        include_str!("../../tests/fixtures/hd_capture.mdl"),
    ] {
        let source = Wc3Model::decode_mdl(text).unwrap();
        let mut meshes = Assets::default();
        let mut bindposes = Assets::default();
        let prepared = prepare_model(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
        // Projections into the source would increase this count even if the
        // PreparedModel struct itself no longer had a model field.
        assert_eq!(source.model.strong_count(), 1);
        drop(source);
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        let root = spawn_prepared_model(
            &mut commands,
            &mut meshes,
            &mut Assets::default(),
            &prepared,
        );
        queue.apply(&mut world);
        assert!(world.get::<Wc3Animation>(root).is_some());
        Wc3PoseBaker::default()
            .bake(
                &prepared,
                &meshes,
                &mut Assets::default(),
                &Wc3TextureBindings::default(),
                Wc3PoseOptions {
                    camera: Some(Transform::IDENTITY),
                    ..default()
                },
            )
            .unwrap();
    }
}
