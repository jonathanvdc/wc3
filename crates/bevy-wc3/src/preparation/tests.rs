use super::*;
use crate::animation::Wc3Animation;
use crate::assets::DecodedDocument;
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
        let source = Wc3Document::decode_mdl(text).unwrap();
        let mut meshes = Assets::default();
        let mut bindposes = Assets::default();
        let prepared = prepare_document(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
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

#[test]
fn mdx_geometry_borrows_input_and_compilation_outlives_the_bytes() {
    let compiled = {
        let bytes = include_bytes!("../../tests/fixtures/quad_model.mdx").to_vec();
        let decoded = DecodedDocument::decode(&bytes).unwrap();
        let geometry = decoded.model.geosets().next().unwrap();
        let pointer = geometry.vertices().as_bytes().as_ptr() as usize;
        let start = bytes.as_ptr() as usize;
        assert!((start..start + bytes.len()).contains(&pointer));
        compile_document(decoded.model, |_| None, |_| None).unwrap()
    };
    let mut meshes = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut Assets::default());
    drop(compiled);
    let pose = Wc3PoseBaker::default()
        .bake(
            &prepared,
            &meshes,
            &mut Assets::default(),
            &Wc3TextureBindings::default(),
            Wc3PoseOptions::default(),
        )
        .unwrap();
    assert!(!pose.parts.is_empty());
}

#[test]
fn consuming_compilation_moves_owned_tracks_and_shares_feature_node_tracks() {
    let decoded =
        DecodedDocument::decode(include_bytes!("../../tests/fixtures/particle_capture.mdl"))
            .unwrap();
    let pointer = decoded
        .model
        .helpers()
        .next()
        .unwrap()
        .rotation
        .as_ref()
        .unwrap()
        .linear_keys()
        .unwrap()
        .as_bytes()
        .as_ptr();
    let compiled = compile_document(decoded.model, |_| None, |_| None).unwrap();
    assert_eq!(
        compiled.nodes[0]
            .rotation
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()
            .as_bytes()
            .as_ptr(),
        pointer
    );
    // Feature nodes project from the same owned emitter record used by simulation.
    assert_eq!(
        &*compiled.nodes[1].rotation as *const _,
        &compiled.particles2[0].node.rotation as *const _
    );
    let prepared = compiled.prepare(&mut Assets::default(), &mut Assets::default());
    assert_eq!(
        prepared.nodes[0]
            .rotation
            .as_ref()
            .unwrap()
            .linear_keys()
            .unwrap()
            .as_bytes()
            .as_ptr(),
        pointer
    );
}

#[test]
fn invalid_geometry_fails_compilation_before_asset_insertion() {
    let text = include_str!("../../tests/fixtures/quad_model.mdl")
        .replace("Groups 1 1", "Groups 1 9")
        .replace("Matrices { 0 }", "Matrices { 0, 0, 0, 0, 0, 0, 0, 0, 0 }");
    let decoded = DecodedDocument::decode(text.as_bytes()).unwrap();
    assert!(compile_document(decoded.model, |_| None, |_| None).is_err());
}
