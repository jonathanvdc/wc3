use crate::runtime::synchronize_model_cameras;
use crate::{ModelAsset, ModelInstance};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

use super::*;
use std::collections::HashMap;
use wc3::model::profiles::Universal;
use wc3::model::{Model, ModelMetadata};
fn child(app: &mut App, root: Entity, owner: Entity) -> Entity {
    app.world_mut()
        .spawn((
            ModelParticleChild {
                root,
                birth: 1.,
                lifetime: 10.,
                visible: true,
                first_clip_pending: false,
            },
            ModelParticleOwner(owner),
            ModelParticleMotion {
                origin: Vec3::new(3., 4., 5.),
                velocity: Vec3::X * 2.,
                acceleration: Vec3::NEG_Z * 4.,
                rotation: Quat::from_rotation_y(0.7),
                scale: Vec3::splat(0.5),
            },
            Transform::default(),
            Visibility::default(),
            Playback::default(),
        ))
        .id()
}
#[test]
fn delayed_children_follow_particle_age_and_frozen_world_trajectory() {
    let mut slot_values = SlotValues::default();
    let mut app = App::new();
    app.add_systems(
        Update,
        (synchronize_model_particles, synchronize_model_cameras).chain(),
    );
    let camera = app.world_mut().spawn_empty().id();
    let root = app
        .world_mut()
        .spawn((
            Playback {
                shared_time: 3.,
                driving_camera: Some(camera),
                ..SlotConstruct::construct(&mut slot_values)
            },
            Transform::from_xyz(100., 200., 300.),
        ))
        .id();
    let owner = app.world_mut().spawn_empty().id();
    let child = child(&mut app, root, owner);
    app.update();
    assert_eq!(
        app.world().get::<Transform>(child).unwrap().translation,
        Vec3::new(7., 4., -3.)
    );
    let playback = app.world().get::<Playback>(child).unwrap();
    assert_eq!((playback.clip_time, playback.shared_time), (2., 2.));
    assert!(playback.paused);
    assert_eq!(playback.driving_camera, Some(camera));
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .driving_camera = None;
    app.world_mut()
        .get_mut::<Transform>(root)
        .unwrap()
        .translation = Vec3::splat(-500.);
    app.update();
    assert_eq!(
        app.world().get::<Playback>(child).unwrap().driving_camera,
        None
    );
    assert_eq!(
        app.world().get::<Transform>(child).unwrap().translation,
        Vec3::new(7., 4., -3.)
    );
}
#[test]
fn detached_children_are_cleaned_up_with_the_renderer_owner() {
    let mut slot_values = SlotValues::default();
    let mut app = App::new();
    app.add_systems(Update, synchronize_model_particles);
    let root = app
        .world_mut()
        .spawn(Playback {
            shared_time: 2.,
            ..SlotConstruct::construct(&mut slot_values)
        })
        .id();
    let owner = app.world_mut().spawn_empty().id();
    let child = child(&mut app, root, owner);
    app.world_mut().despawn(owner);
    app.update();
    assert!(app.world().get_entity(child).is_err());
}
#[test]
fn detached_particle_requests_cannot_reload_an_ancestor_model() {
    use super::super::super::instances::{spawn_loaded_models, PreparedModelCache};
    use super::super::super::{
        compile_model, ModelInstanceError, ModelMaterial, ModelRig, ModelSource,
    };
    use crate::image_replacements::ReplacementImages;
    use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
    let model = Model::<Universal>::empty(ModelMetadata::Mdlx(Default::default())).unwrap();
    let mut app = App::new();
    app.init_resource::<Assets<ModelAsset>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<SkinnedMeshInverseBindposes>>()
        .init_resource::<Assets<ModelMaterial>>()
        .init_resource::<PreparedModelCache>()
        .init_resource::<ReplacementImages>()
        .init_resource::<Assets<Image>>()
        .add_systems(Update, spawn_loaded_models);
    let handle = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .add(ModelAsset {
            compiled: compile_model(&model, |_| None).unwrap(),
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        });
    let root = app.world_mut().spawn(ModelSource(handle.clone())).id();
    let owner = app.world_mut().spawn_empty().id();
    let particle = child(&mut app, root, owner);
    app.world_mut()
        .entity_mut(particle)
        .insert(ModelInstance(handle));
    app.update();
    assert!(
        matches!(app.world().get::<ModelInstanceError>(particle), Some(ModelInstanceError::CyclicDependency { ancestor, .. }) if *ancestor == root)
    );
    assert!(app.world().get::<ModelRig>(particle).is_none());
}
