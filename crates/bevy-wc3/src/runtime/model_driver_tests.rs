use super::*;
use crate::compile_model;
use crate::effects::model_children::ModelParticleMotion;
use crate::effects::model_ownership::ModelParticleOwner;
use crate::image_replacements::ReplacementImages;
use crate::instances::PreparedModelCache;
use crate::{ModelAsset, ModelMaterial};
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use std::collections::HashMap;
use wc3::model::{profiles::Universal, Model, ModelMetadata};

fn scene() -> (App, Entity, Handle<ModelAsset>) {
    let model = Model::<Universal>::empty(ModelMetadata::Mdlx(Default::default())).unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut materials = Assets::<ModelMaterial>::default();
    let mut app = App::new();
    let mut queue = CommandQueue::default();
    let root = super::super::spawn_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::from_xyz(80., 90., 100.),
    );
    queue.apply(app.world_mut());
    app.world_mut()
        .entity_mut(root)
        .insert(ModelParticleDriver::default());
    let mut assets = Assets::default();
    let handle = assets.add(ModelAsset {
        compiled,
        images: HashMap::new(),
        unresolved_images: Vec::new(),
    });
    app.insert_resource(meshes)
        .insert_resource(binds)
        .insert_resource(materials)
        .insert_resource(assets)
        .init_resource::<Assets<Image>>()
        .init_resource::<PreparedModelCache>()
        .init_resource::<ReplacementImages>()
        .init_resource::<ModelParticleBirths>()
        .add_systems(Update, simulate_model_particles);
    (app, root, handle)
}
fn birth(app: &mut App, root: Entity, handle: Handle<ModelAsset>, lifetime: f64) -> Entity {
    app.world_mut()
        .spawn((
            ModelInstance(handle),
            ModelParticleOwner(root),
            ModelParticleChild {
                root,
                birth: 0.,
                lifetime,
                visible: true,
                first_clip_pending: true,
            },
            ModelParticleMotion {
                origin: Vec3::new(3., 4., 5.),
                velocity: Vec3::X * 2.,
                acceleration: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Playback {
                paused: true,
                ..Default::default()
            },
            Transform::default(),
            Visibility::Inherited,
        ))
        .id()
}
#[test]
fn ready_birth_shorter_than_frame_is_populated_and_sampled_in_same_step() {
    let (mut app, root, handle) = scene();
    let child = birth(&mut app, root, handle, 0.005);
    app.update();
    assert!(
        app.world().get::<ModelRig>(child).is_some(),
        "ready birth must not wait until next Update"
    );
    assert!(app.world().get::<ModelInstance>(child).is_none());
    assert_eq!(
        app.world().get::<Transform>(child).unwrap().translation,
        Vec3::new(3., 4., 5.)
    );
    assert_eq!(app.world().get::<Playback>(child).unwrap().shared_time, 0.);
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .shared_time = 0.006;
    app.update();
    assert!(app.world().get_entity(child).is_err());
}
#[test]
fn visibility_and_camera_inheritance_use_current_ancestors_before_propagation() {
    let (mut app, root, handle) = scene();
    let ancestor = app
        .world_mut()
        .spawn((Visibility::Hidden, Transform::default()))
        .id();
    app.world_mut().entity_mut(root).insert(ChildOf(ancestor));
    let camera = app.world_mut().spawn(Transform::from_xyz(8., 3., 1.)).id();
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .driving_camera = Some(camera);
    let child = birth(&mut app, root, handle, 10.);
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(child).unwrap(),
        Visibility::Hidden
    );
    assert_eq!(
        app.world().get::<Playback>(child).unwrap().driving_camera,
        Some(camera)
    );
    *app.world_mut().get_mut::<Visibility>(ancestor).unwrap() = Visibility::Visible;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(child).unwrap(),
        Visibility::Inherited
    );
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .driving_camera = None;
    *app.world_mut().get_mut::<Visibility>(ancestor).unwrap() = Visibility::Hidden;
    *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Visible;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(child).unwrap(),
        Visibility::Inherited
    );
    assert_eq!(
        app.world().get::<Playback>(child).unwrap().driving_camera,
        None
    );
}
#[test]
fn pending_birth_dies_before_loading_and_cannot_be_populated_later() {
    let (mut app, root, _) = scene();
    let pending = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .reserve_handle();
    let child = birth(&mut app, root, pending, 0.005);
    app.update();
    assert!(app.world().get::<ModelInstance>(child).is_some());
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .shared_time = 0.006;
    app.update();
    assert!(app.world().get_entity(child).is_err());
}
