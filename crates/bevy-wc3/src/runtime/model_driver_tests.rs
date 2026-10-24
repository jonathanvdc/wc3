use super::*;
use crate::compile_model;
use crate::effects::model_children::ModelParticleMotion;
use crate::effects::model_ownership::ModelParticleOwner;
use crate::image_replacements::ReplacementImages;
use crate::instances::PreparedModelCache;
use crate::{ModelAsset, ModelMaterial, PreparedGeometryCache};
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
    app.add_plugins(bevy::diagnostic::FrameCountPlugin);
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
        .init_resource::<PreparedGeometryCache>()
        .init_resource::<ReplacementImages>()
        .init_resource::<ModelParticleBirths>()
        .add_systems(Update, simulate_model_particles);
    // Establish the ordinary application root's initial effect-clock origin.
    app.update();
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

fn effect_asset(app: &mut App, source: &str) -> Handle<ModelAsset> {
    use wc3::model::{conversion::ImportOptions, io::mdlx};
    let model = mdlx::decode_mdl(source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    let compiled = compile_model(&model, |_| Some(image.clone())).unwrap();
    app.world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .add(ModelAsset {
            compiled,
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        })
}
fn enable_effect_publication(app: &mut App) {
    use bevy::transform::{TransformPlugin, TransformSystems};
    app.add_plugins(TransformPlugin).add_systems(
        PostUpdate,
        super::super::simulate_particles.after(TransformSystems::Propagate),
    );
}
fn start_child(app: &mut App, root: Entity, handle: Handle<ModelAsset>) -> Entity {
    let child = birth(app, root, handle, 10.);
    app.world_mut()
        .entity_mut(child)
        .insert(ModelParticleStart {
            basis: Mat4::from_translation(Vec3::new(3., 4., 5.)),
            source: root,
            observed_time: None,
            interval: 0.,
        });
    child
}
fn sprite_population(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<&crate::effects::SpriteEmitter>();
    query.iter(app.world()).map(|value| value.alive).sum()
}
fn ribbon_population(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<&crate::effects::RibbonEmitter>();
    query.iter(app.world()).map(|value| value.sections).sum()
}
#[test]
fn ready_child_sprites_and_ribbons_integrate_the_first_subframe_once() {
    for (source, population) in [
        (
            include_str!("../../tests/fixtures/particle_capture.mdl"),
            sprite_population as fn(&mut App) -> usize,
        ),
        (
            include_str!("../../tests/fixtures/ribbon_capture.mdl"),
            ribbon_population,
        ),
    ] {
        let (mut app, root, _) = scene();
        enable_effect_publication(&mut app);
        let handle = effect_asset(&mut app, source);
        let child = start_child(&mut app, root, handle.clone());
        app.world_mut()
            .get_mut::<Playback>(root)
            .unwrap()
            .shared_time = 0.1;
        app.update();
        let first = population(&mut app);
        assert!(
            first > 0,
            "ready child must integrate 0..0.1 rather than start at its endpoint"
        );
        assert!(app.world().get::<ModelParticleStart>(child).is_none());
        app.update();
        assert_eq!(
            population(&mut app),
            first,
            "paused frames cannot replay startup births"
        );
        // Replacing an already populated asset has no birth token. Its history
        // starts at the present time, regardless of the owning particle's age.
        app.world_mut()
            .entity_mut(child)
            .insert(ModelInstance(handle));
        app.update();
        assert_eq!(
            population(&mut app),
            0,
            "reload must not recreate old effects"
        );
    }
}
#[test]
fn pending_child_startup_survives_paused_loading_but_expires_when_age_advances() {
    for advance in [false, true] {
        let (mut app, root, _) = scene();
        enable_effect_publication(&mut app);
        let ready = effect_asset(
            &mut app,
            include_str!("../../tests/fixtures/particle_capture.mdl"),
        );
        let asset = app
            .world()
            .resource::<Assets<ModelAsset>>()
            .get(&ready)
            .unwrap();
        let asset = ModelAsset {
            compiled: asset.compiled.clone(),
            images: asset.images.clone(),
            unresolved_images: asset.unresolved_images.clone(),
        };
        let pending = app
            .world_mut()
            .resource_mut::<Assets<ModelAsset>>()
            .reserve_handle();
        let child = start_child(&mut app, root, pending.clone());
        app.world_mut()
            .get_mut::<Playback>(root)
            .unwrap()
            .shared_time = 0.1;
        app.update();
        assert!(app.world().get::<ModelParticleStart>(child).is_some());
        if advance {
            app.world_mut()
                .get_mut::<Playback>(root)
                .unwrap()
                .shared_time = 0.2;
        }
        app.world_mut()
            .resource_mut::<Assets<ModelAsset>>()
            .insert(pending.id(), asset)
            .unwrap();
        app.update();
        assert_eq!(
            sprite_population(&mut app) > 0,
            !advance,
            "only the unchanged first observed age may replay its initial subframe"
        );
        assert!(app.world().get::<ModelParticleStart>(child).is_none());
    }
}

#[test]
fn independent_attachment_start_applies_only_its_own_speed_once() {
    for (speed, paused, expected) in [(1., false, 0.1), (2., false, 0.2), (2., true, 0.)] {
        let mut clock = Playback {
            speed,
            paused,
            ..Default::default()
        };
        assert!(seed_independent_clock(&mut clock, 0.1));
        assert_eq!((clock.clip_time, clock.shared_time), (expected, expected));
        assert_eq!(clock.speed, speed);
        assert_eq!(clock.paused, paused);
    }
    let mut clock = Playback {
        shared_time: 3.,
        clip_time: 2.,
        ..Default::default()
    };
    assert!(!seed_independent_clock(&mut clock, 0.1));
    assert_eq!((clock.clip_time, clock.shared_time), (2., 3.));
    assert!(!seed_independent_clock(&mut Playback::default(), -1.));
}
#[test]
fn only_constant_active_fresh_attachments_receive_an_initial_interval() {
    use crate::child_coordinates::CoordinatePolicy;
    for (visibility, expected) in [
        ("", true),
        ("Visibility 1 { DontInterp, 0: 0.0, }", false),
        ("Visibility 2 { Linear, 0: 1.0, 1000: 1.0, }", false),
    ] {
        let source = format!(
            r#"
            Version {{ FormatVersion 800, }}
            Model "startup" {{}}
            Sequences 1 {{ Anim "Stand" {{ Interval {{ 0, 1000 }}, }} }}
            Attachment "socket" {{ ObjectId 0, AttachmentID 0, {visibility} }}
            PivotPoints 1 {{ {{ 0., 0., 0. }}, }}
        "#
        );
        let (mut app, root, _) = scene();
        let handle = effect_asset(&mut app, &source);
        let parent = birth(&mut app, root, handle, 10.);
        app.update();
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<ModelAttachment>>();
        let point = query.single(app.world()).unwrap();
        app.world_mut().entity_mut(point).insert((
            ChildCoordinates {
                coordinates: CoordinatePolicy::Host,
                parent_basis: Transform::IDENTITY,
            },
            Playback {
                speed: 2.,
                ..Default::default()
            },
        ));
        app.world_mut().get_mut::<Playback>(parent).unwrap().speed = 7.;
        let mut pipeline = ModelPipeline::new(app.world_mut());
        pipeline.birth_roots.insert(
            parent,
            BirthActor {
                root: Mat4::IDENTITY,
                basis: Mat4::IDENTITY,
                camera: None,
                elapsed: 0.1,
            },
        );
        assert_eq!(
            initialize_attachment_start(app.world_mut(), &mut pipeline, point),
            expected
        );
        assert_eq!(
            app.world().get::<Playback>(point).unwrap().shared_time,
            if expected { 0.2 } else { 0. }
        );
        if expected {
            app.world_mut()
                .entity_mut(point)
                .remove::<(Playback, ModelParticleStart)>();
            assert!(!initialize_attachment_start(
                app.world_mut(),
                &mut pipeline,
                point
            ));
            assert!(
                app.world().get::<ModelParticleStart>(point).is_some(),
                "pending attachment captures the same startup token as direct children"
            );
            pipeline.birth_roots.clear();
            app.world_mut().entity_mut(point).insert(Playback {
                speed: 3.,
                ..Default::default()
            });
            assert!(
                initialize_attachment_start(app.world_mut(), &mut pipeline, point),
                "unchanged owner age retains paused loading history"
            );
            assert!((app.world().get::<Playback>(point).unwrap().shared_time - 0.3).abs() < 1e-10);
            app.world_mut()
                .get_mut::<Playback>(parent)
                .unwrap()
                .shared_time = 0.2;
            app.world_mut()
                .entity_mut(point)
                .insert(Playback::default());
            assert!(!initialize_attachment_start(
                app.world_mut(),
                &mut pipeline,
                point
            ));
            assert!(app.world().get::<ModelParticleStart>(point).is_none());
        }
        // A later ready load or reload has no same-stage parent birth entry.
        pipeline.birth_roots.clear();
        assert!(!initialize_attachment_start(
            app.world_mut(),
            &mut pipeline,
            point
        ));
    }
}

#[test]
fn global_loader_ready_attachment_consumes_its_captured_interval_before_pose() {
    use crate::child_coordinates::CoordinatePolicy;
    use crate::instances::spawn_loaded_models;
    let (mut app, root, _) = scene();
    enable_effect_publication(&mut app);
    let parent_asset = effect_asset(
        &mut app,
        r#"
        Version { FormatVersion 800, }
        Model "startup" {}
        Sequences 1 { Anim "Stand" { Interval { 0, 1000 }, } }
        Attachment "socket" { ObjectId 0, AttachmentID 0, }
        PivotPoints 1 { { 0., 0., 0. }, }
    "#,
    );
    let parent = start_child(&mut app, root, parent_asset);
    let mut factory = SystemState::<OwnedModelFactory>::new(app.world_mut());
    factory.get_mut(app.world_mut()).unwrap().populate(parent);
    factory.apply(app.world_mut());
    let mut points = app
        .world_mut()
        .query_filtered::<Entity, With<ModelAttachment>>();
    let point = points.single(app.world()).unwrap();
    let pending = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .reserve_handle();
    app.world_mut().entity_mut(point).insert((
        ModelInstance(pending.clone()),
        ParticleModelSubtree,
        ChildCoordinates {
            coordinates: CoordinatePolicy::Host,
            parent_basis: Transform::IDENTITY,
        },
    ));
    app.world_mut()
        .resource_mut::<ModelParticleBirths>()
        .0
        .push_back(point);
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .shared_time = 0.1;
    app.update();
    assert!(app.world().get::<Playback>(point).is_none());
    assert!(app.world().get::<ModelParticleStart>(point).is_some());
    let ready = effect_asset(
        &mut app,
        include_str!("../../tests/fixtures/particle_capture.mdl"),
    );
    let asset = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .remove(ready.id())
        .unwrap();
    app.world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .insert(pending.id(), asset)
        .unwrap();
    app.add_systems(Update, spawn_loaded_models.before(simulate_model_particles));
    app.update();
    assert!((app.world().get::<Playback>(point).unwrap().shared_time - 0.1).abs() < 1e-10);
    assert!(
        sprite_population(&mut app) > 0,
        "global readiness must not erase the accepted initial interval"
    );
    assert!(app.world().get::<ModelParticleStart>(point).is_none());
    let count = sprite_population(&mut app);
    app.update();
    assert_eq!(sprite_population(&mut app), count);
}

#[test]
fn static_owned_descendants_share_admission_limit_and_release_with_owner() {
    use crate::compiled::RuntimeMount;
    use crate::effects::model_budget::{
        ActorBudget, ModelActorTree, ModelParticleLimits, ModelParticleTreeStats,
    };
    use bevy::diagnostic::FrameCount;
    let (mut app, root, _) = scene();
    let handle = effect_asset(
        &mut app,
        r#"Version { FormatVersion 800, } Model "static" {} Bone "pivot" { ObjectId 0, GeosetId Multiple, GeosetAnimId None, } PivotPoints 1 { { 0, 0, 0 }, }"#,
    );
    let assets = app.world().resource::<Assets<ModelAsset>>();
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let leaf = assets
        .get(&handle)
        .unwrap()
        .compiled
        .prepare(&mut meshes, &mut binds);
    let mut branch = leaf.clone();
    branch.mounts = (0..5)
        .map(|_| RuntimeMount {
            parent: 0,
            socket: None,
            transform: Transform::default(),
            model: leaf.clone(),
            clips: HashMap::new(),
        })
        .collect();
    let limits = ModelParticleLimits {
        actors_per_tree: 3,
        birth_work_per_frame: 20,
        ..Default::default()
    };
    let budget = ActorBudget::new(root, limits);
    app.world_mut().entity_mut(root).insert((
        limits,
        ModelActorTree {
            budget: budget.clone(),
            pending: Default::default(),
        },
    ));
    let frame = *app.world().resource::<FrameCount>();
    let child = birth(&mut app, root, handle, 10.);
    app.world_mut()
        .entity_mut(child)
        .remove::<ModelInstance>()
        .insert((
            PreparedModelRequest(branch),
            budget.reserve(frame).unwrap(),
            ParticleModelSubtree,
        ));
    app.update();
    assert!(app.world().get::<ModelRig>(child).is_some());
    let stats = app.world().get::<ModelParticleTreeStats>(root).unwrap();
    assert_eq!((stats.actors, stats.pending), (3, 0));
    assert_eq!(
        stats.omitted_actor_limit, 3,
        "static mounts count even without emitters"
    );
    let children = app.world().get::<ModelRig>(child).unwrap().mounts();
    assert_eq!(children.len(), 2);
    for entity in children {
        assert!(
            app.world().get::<ModelRig>(*entity).is_some(),
            "ready static descendants populate through the bounded worklist"
        );
    }
    app.world_mut().despawn(child);
    assert_eq!(budget.stats(frame).actors, 0);
}

fn model_emitter_asset(app: &mut App, child: Handle<ModelAsset>, rate: f32) -> Handle<ModelAsset> {
    use crate::{compile_model_with_options, ChildModelBinding, ChildModelKey, CompileOptions};
    use wc3::model::{conversion::ImportOptions, io::mdlx};
    let source = format!(
        r#"Version {{ FormatVersion 800, }} Model "nested" {{}} ParticleEmitter "source" {{ ObjectId 0, EmitterUsesMdl, Path "child.mdl", static EmissionRate {rate}, static Gravity 0, static Longitude 0, static Latitude 0, static LifeSpan 2, static InitVelocity 0, }} PivotPoints 1 {{ {{ 0,0,0 }}, }}"#
    );
    let model = mdlx::decode_mdl(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let feature = model.features().iter().next().unwrap().0;
    let compiled = compile_model_with_options(
        &model,
        CompileOptions {
            child_models: [(
                ChildModelKey::particle_geometry(feature),
                ChildModelBinding::new(child),
            )]
            .into(),
            ..Default::default()
        },
        |_| None,
    )
    .unwrap();
    app.world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .add(ModelAsset {
            compiled,
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        })
}
#[test]
fn nested_model_births_share_tree_limits_and_never_replay_a_rejected_interval() {
    use crate::effects::model_budget::{ModelParticleLimits, ModelParticleTreeStats};
    let (mut app, root, leaf) = scene();
    let middle = model_emitter_asset(&mut app, leaf, 100.);
    let outer = model_emitter_asset(&mut app, middle, 10.);
    app.world_mut().entity_mut(root).insert((
        ModelInstance(outer),
        ModelParticleLimits {
            actors_per_tree: 5,
            birth_work_per_frame: 20,
            emission_steps_per_frame: 40,
        },
    ));
    app.update();
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .shared_time = 0.25;
    app.update();
    let stats = app.world().get::<ModelParticleTreeStats>(root).unwrap();
    assert_eq!(stats.actors, 5);
    assert_eq!(stats.pending, 0);
    assert!(stats.omitted_actor_requests() > 0);
    assert!(stats.birth_work <= 20 && stats.emission_steps <= 40);
    let mut children = app.world_mut().query::<&ModelParticleChild>();
    assert!(
        children.iter(app.world()).any(|child| child.root != root),
        "a second geometry-model generation must actually be admitted"
    );
    let total = stats_actors(&app, root);
    app.update();
    assert_eq!(
        stats_actors(&app, root),
        total,
        "paused readiness cannot replay the omitted source interval"
    );
    app.world_mut().get_mut::<Playback>(root).unwrap().restart();
    app.update();
    assert_eq!(
        stats_actors(&app, root),
        0,
        "reset releases the entire ownership tree"
    );
}
fn stats_actors(app: &App, root: Entity) -> usize {
    app.world()
        .get::<crate::effects::model_budget::ModelParticleTreeStats>(root)
        .unwrap()
        .actors
}

#[test]
fn stale_leased_ready_request_is_rejected_before_factory_population() {
    use crate::effects::model_budget::{ActorBudget, ModelActorTree, ModelParticleLimits};
    use bevy::diagnostic::FrameCount;
    let (mut app, root, handle) = scene();
    let old = ActorBudget::new(root, ModelParticleLimits::default());
    let prepared = app.world().get::<ModelRig>(root).unwrap().model().clone();
    let child = birth(&mut app, root, handle, 1.);
    app.world_mut()
        .entity_mut(child)
        .remove::<ModelInstance>()
        .insert((
            PreparedModelRequest(prepared),
            old.reserve(FrameCount(0)).unwrap(),
            ParticleModelSubtree,
        ));
    let new = ActorBudget::new(root, ModelParticleLimits::default());
    app.world_mut().entity_mut(root).insert(ModelActorTree {
        budget: new.clone(),
        pending: Default::default(),
    });
    app.update();
    assert!(app.world().get_entity(child).is_err());
    assert_eq!(old.stats(FrameCount(0)).actors, 0);
    assert_eq!(new.stats(FrameCount(0)).actors, 0);
}
