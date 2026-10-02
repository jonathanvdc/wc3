use super::*;
use crate::animation::pose::animate_nodes;
use bevy::camera::visibility::VisibilityPlugin;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::transform::TransformSystems;
use std::time::Duration;
use wc3::model::scene::Node;

use crate::animation::advance_animation;
use crate::assets::model::Wc3Model;
use crate::instance::{spawn_loaded_instances, PreparedModelCache, Wc3OwnedModels};
use crate::materials::layers::animate_layers;
use crate::materials::Wc3LayerMaterial;

fn setup() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((TransformPlugin, VisibilityPlugin));
    app.insert_resource(Time::<()>::default());
    app.init_resource::<Assets<Wc3ModelAsset>>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<Wc3LayerMaterial>>();
    app.init_resource::<Assets<SkinnedMeshInverseBindposes>>();
    app.init_resource::<PreparedModelCache>();
    app.add_systems(
        Update,
        (
            spawn_loaded_instances,
            advance_animation,
            animate_particle_models,
            animate_nodes,
            animate_layers,
        )
            .chain(),
    );
    app.add_systems(
        PostUpdate,
        update_particles.before(TransformSystems::Propagate),
    );
    let source = Wc3Model::decode_mdl(include_str!(
        "../../../tests/fixtures/attachment_capture_child.mdl"
    ))
    .unwrap();
    let model = app
        .world_mut()
        .resource_mut::<Assets<Wc3ModelAsset>>()
        .add(Wc3ModelAsset {
            source,
            textures: default(),
            models: default(),
        });
    let root = app
        .world_mut()
        .spawn((
            Wc3Animation {
                sequence: 0,
                elapsed_ms: 0.0,
                speed: 1.0,
                playing: true,
                sequences: Vec::new(),
                global_sequences: Vec::new(),
                event_playback: Default::default(),
            },
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    let node = app
        .world_mut()
        .spawn((Transform::from_xyz(2.0, 0.0, 0.0), ChildOf(root)))
        .id();
    let mut definition =
        ParticleEmitter::new(Node::new("Emitter", 0).unwrap(), "child.mdl").unwrap();
    definition.emission_rate = 4.0.into();
    definition.life_span = 1.0.into();
    definition.initial_velocity = 4.0.into();
    definition.gravity = 2.0.into();
    let emitter = app
        .world_mut()
        .spawn((
            ParticleState::new(root, node, definition, model),
            ChildOf(root),
        ))
        .id();
    step(&mut app, 0.0);
    (app, root, emitter)
}

fn step(app: &mut App, seconds: f64) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(seconds));
    app.update();
}

#[test]
fn births_are_subframe_and_children_have_independent_clocks() {
    let (mut app, _, emitter) = setup();
    step(&mut app, 0.625);
    let state = app.world().get::<ParticleState>(emitter).unwrap();
    assert_eq!(state.particles.len(), 2);
    let first = state.particles[0].entity;
    let second = state.particles[1].entity;
    assert_eq!(state.particles[0].age, 0.375);
    assert_eq!(state.particles[1].age, 0.125);
    let transform = app.world().get::<Transform>(first).unwrap();
    assert!((transform.translation.z - 1.359375).abs() < 1e-6);
    assert_eq!(transform.translation.x, 2.0);
    assert!(app.world().get::<ChildOf>(first).is_none());
    // Zero-time render warmup loads models without advancing their age.
    step(&mut app, 0.0);
    assert_eq!(
        app.world().get::<Wc3Animation>(first).unwrap().elapsed_ms,
        375.0
    );
    assert_eq!(
        app.world().get::<Wc3Animation>(second).unwrap().elapsed_ms,
        125.0
    );
    assert_eq!(
        app.world()
            .resource::<Assets<SkinnedMeshInverseBindposes>>()
            .len(),
        1
    );
}

#[test]
fn world_motion_pause_speed_expiry_and_owner_cleanup() {
    let (mut app, root, emitter) = setup();
    step(&mut app, 0.25);
    let particle = app.world().get::<ParticleState>(emitter).unwrap().particles[0].entity;
    app.world_mut()
        .get_mut::<Transform>(root)
        .unwrap()
        .translation
        .x = 10.0;
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .playing = false;
    step(&mut app, 0.5);
    assert_eq!(
        app.world().get::<Transform>(particle).unwrap().translation,
        Vec3::new(2.0, 0.0, 0.0)
    );
    assert_eq!(
        app.world()
            .get::<Wc3Animation>(particle)
            .unwrap()
            .elapsed_ms,
        0.0
    );
    {
        let mut animation = app.world_mut().get_mut::<Wc3Animation>(root).unwrap();
        animation.playing = true;
        animation.speed = 2.0;
    }
    step(&mut app, 0.25);
    assert_eq!(
        app.world().get::<Transform>(particle).unwrap().translation,
        Vec3::new(2.0, 0.0, 1.75)
    );
    assert_eq!(
        app.world()
            .get::<Wc3Animation>(particle)
            .unwrap()
            .elapsed_ms,
        500.0
    );
    step(&mut app, 0.25);
    assert!(app.world().get_entity(particle).is_err());
    let owned: Vec<_> = app
        .world()
        .get::<Wc3OwnedModels>(root)
        .unwrap()
        .iter()
        .collect();
    assert!(!owned.is_empty());
    app.world_mut().despawn(root);
    assert!(owned
        .into_iter()
        .all(|entity| app.world().get_entity(entity).is_err()));
    assert!(app.world().get_entity(emitter).is_err());
}

#[test]
fn hidden_emitter_stops_births_but_existing_particles_age() {
    let (mut app, root, emitter) = setup();
    step(&mut app, 0.25);
    app.world_mut()
        .get_mut::<Visibility>(root)
        .unwrap()
        .clone_from(&Visibility::Hidden);
    step(&mut app, 0.25);
    let state = app.world().get::<ParticleState>(emitter).unwrap();
    assert_eq!(state.particles.len(), 1);
    assert_eq!(state.particles[0].age, 0.25);
}

#[test]
fn backward_seek_retires_old_particles() {
    let (mut app, root, emitter) = setup();
    step(&mut app, 0.5);
    let old: Vec<_> = app
        .world()
        .get::<ParticleState>(emitter)
        .unwrap()
        .particles
        .iter()
        .map(|p| p.entity)
        .collect();
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .elapsed_ms = 0.0;
    step(&mut app, 0.0);
    assert!(app
        .world()
        .get::<ParticleState>(emitter)
        .unwrap()
        .particles
        .is_empty());
    assert!(old
        .into_iter()
        .all(|entity| app.world().get_entity(entity).is_err()));
}
#[test]
fn moving_nodes_are_sampled_at_birth() {
    let (mut app, root, emitter) = setup();
    let node = app.world().get::<ParticleState>(emitter).unwrap().node;
    let source =
        Wc3Model::decode_mdl(include_str!("../../../tests/fixtures/prem_capture.mdl")).unwrap();
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .sequences = source.model.sequences();
    let definition = source.model.particle_emitters().remove(0);
    app.world_mut().entity_mut(node).insert(AnimatedNode {
        flags: Default::default(),
        camera: None,
        root,
        pivot: Vec3::ZERO,
        parent_pivot: Vec3::ZERO,
        translation: definition.node.translation,
        rotation: None,
        scaling: None,
    });
    step(&mut app, 0.625);
    let state = app.world().get::<ParticleState>(emitter).unwrap();
    assert_eq!(state.particles.len(), 2);
    assert_eq!(state.particles[0].origin.x, -1.5);
    assert_eq!(state.particles[1].origin.x, 0.0);
}

#[test]
fn recursive_particle_models_are_blocked() {
    use crate::instance::BlockedChildModel;
    let (mut app, root, emitter) = setup();
    let handle = app
        .world()
        .get::<ParticleState>(emitter)
        .unwrap()
        .model
        .clone();
    app.world_mut()
        .entity_mut(root)
        .insert(Wc3ModelInstance::new(handle));
    step(&mut app, 0.25);
    let child = app.world().get::<ParticleState>(emitter).unwrap().particles[0].entity;
    step(&mut app, 0.0);
    assert!(app.world().get::<BlockedChildModel>(child).is_some());
    assert!(app.world().get::<Wc3Animation>(child).is_none());
}
