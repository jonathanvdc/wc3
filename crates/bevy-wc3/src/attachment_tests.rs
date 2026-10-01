use super::*;
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use wc3::model::animation::{Sequence, ValueKeyframe};
use wc3::model::scene::{Attachment, Node};

use crate::animation::{animate_layers, animate_nodes, AnimatedLayer};
use crate::asset::ResolvedModelTextures;
use crate::instance::{spawn_loaded_instances, PreparedModelCache};
use crate::material::Wc3LayerMaterial;
use crate::model::Wc3Model;
use crate::model_resources::Wc3ModelResources;
use crate::spawn::Wc3NodeEntities;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, VisibilityPlugin));
    app.insert_resource(Assets::<Wc3ModelAsset>::default());
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<Wc3LayerMaterial>::default());
    app.insert_resource(Assets::<SkinnedMeshInverseBindposes>::default());
    app.init_resource::<PreparedModelCache>();
    app.add_systems(
        Update,
        (
            spawn_loaded_instances,
            animate_attachments,
            animate_nodes,
            animate_layers,
        )
            .chain(),
    );
    app
}

fn asset(source: Wc3Model, models: Wc3ModelResources) -> Wc3ModelAsset {
    Wc3ModelAsset {
        source,
        models,
        textures: ResolvedModelTextures::default(),
    }
}

fn parent_source() -> Wc3Model {
    Wc3Model::decode_mdl(include_str!("../tests/fixtures/attachment_capture.mdl")).unwrap()
}

fn child_source() -> Wc3Model {
    Wc3Model::decode_mdl(include_str!(
        "../tests/fixtures/attachment_capture_child.mdl"
    ))
    .unwrap()
}

fn points(app: &App, root: Entity) -> Vec<Wc3AttachmentPoint> {
    app.world()
        .get::<Wc3Attachments>(root)
        .unwrap()
        .iter()
        .cloned()
        .collect()
}

fn seek(app: &mut App, root: Entity, time: f64) {
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .elapsed_ms = time;
    app.update();
}

#[test]
fn paths_and_consumer_models_follow_points_visibility_and_independent_animation() {
    let mut app = app();
    let child = app
        .world_mut()
        .resource_mut::<Assets<Wc3ModelAsset>>()
        .add(asset(child_source(), default()));
    let source = parent_source();
    let resources = Wc3ModelResources::resolve(&source.model, |_| Some(child.clone()));
    let parent = app
        .world_mut()
        .resource_mut::<Assets<Wc3ModelAsset>>()
        .add(asset(source, resources));
    let root = app
        .world_mut()
        .spawn((
            Wc3ModelInstance::new(parent),
            Transform::from_xyz(10.0, 0.0, 0.0).with_scale(Vec3::new(2.0, 3.0, 1.5)),
        ))
        .id();
    app.update();
    app.update();
    let lookup = app.world().get::<Wc3Attachments>(root).unwrap();
    assert_eq!(lookup.iter().count(), 3);
    assert_eq!(lookup.get(0).unwrap().id, 7);
    assert_eq!(
        lookup.by_id(7).unwrap().node,
        app.world()
            .get::<Wc3NodeEntities>(root)
            .unwrap()
            .get(2)
            .unwrap()
    );
    assert_eq!(lookup.by_name("wEaPoN rEf").unwrap().id, 7);
    assert!(lookup.by_id(2).is_none());
    assert!(lookup.by_name("missing").is_none());
    let empty = lookup.by_id(42).unwrap().clone();
    assert!(empty.model.is_none());
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let supplied = empty.spawn_model(&mut commands, child.clone());
    queue.apply(app.world_mut());
    app.update();
    let points = points(&app, root);
    let weapon = points[0].model.unwrap();
    let shield = points[1].model.unwrap();
    assert_eq!(
        app.world().get::<ChildOf>(weapon).unwrap().parent(),
        points[0].mount
    );
    assert_eq!(
        app.world()
            .get::<ChildOf>(points[0].mount)
            .unwrap()
            .parent(),
        points[0].node
    );
    assert_eq!(app.world().get::<Wc3ModelOwner>(weapon).unwrap().0, root);
    for child_root in [weapon, shield, supplied] {
        assert_eq!(
            app.world().get::<Wc3ModelInstance>(child_root).unwrap().0,
            child
        );
        assert!(!app
            .world()
            .get::<Wc3Animation>(child_root)
            .unwrap()
            .sequences()[0]
            .flags
            .non_looping());
    }
    assert!(app
        .world()
        .resource::<Assets<Wc3ModelAsset>>()
        .get(&child)
        .unwrap()
        .source()
        .model
        .sequences()[0]
        .flags
        .non_looping());
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 2);
    let mut layers = app
        .world_mut()
        .query::<(&AnimatedLayer, &MeshMaterial3d<Wc3LayerMaterial>)>();
    let handles: Vec<_> = layers
        .iter(app.world())
        .filter(|(layer, _)| layer.root == weapon || layer.root == shield)
        .map(|(_, material)| material.0.clone())
        .collect();
    assert_eq!(handles.len(), 2);
    assert_ne!(handles[0], handles[1]);

    seek(&mut app, root, 250.0);
    app.world_mut()
        .get_mut::<Wc3Animation>(weapon)
        .unwrap()
        .elapsed_ms = 600.0;
    app.update();
    let bone = app
        .world()
        .get::<Wc3NodeEntities>(weapon)
        .unwrap()
        .get(0)
        .unwrap();
    let expected = app
        .world()
        .get::<GlobalTransform>(points[0].mount)
        .unwrap()
        .transform_point(Vec3::new(0.0, 0.0, 0.8));
    let actual = app
        .world()
        .get::<GlobalTransform>(bone)
        .unwrap()
        .translation();
    assert!(
        actual.abs_diff_eq(expected, 1e-5),
        "{actual:?} != {expected:?}"
    );
    assert_eq!(
        app.world().get::<Wc3Animation>(shield).unwrap().elapsed_ms,
        0.0
    );
    seek(&mut app, root, 750.0);
    for (point, child_root) in points.iter().zip([weapon, shield, supplied]) {
        assert_eq!(
            app.world()
                .get::<InheritedVisibility>(child_root)
                .unwrap()
                .get(),
            point.id == 42
        );
        assert_eq!(
            app.world().get::<Wc3Animation>(child_root).unwrap().playing,
            point.id == 42
        );
    }
    // The attachment track must not hide its source node or unrelated rig content.
    assert!(app
        .world()
        .get::<InheritedVisibility>(points[0].node)
        .unwrap()
        .get());
    seek(&mut app, root, 1000.0);
    for child_root in [weapon, shield] {
        assert!(app.world().get::<Wc3Animation>(child_root).unwrap().playing);
        assert_eq!(
            app.world()
                .get::<Wc3Animation>(child_root)
                .unwrap()
                .elapsed_ms,
            0.0
        );
    }
    seek(&mut app, root, 400.0);
    assert!(app
        .world()
        .get::<InheritedVisibility>(weapon)
        .unwrap()
        .get());
    assert!(!app
        .world()
        .get::<InheritedVisibility>(shield)
        .unwrap()
        .get());
    // Switching to a sequence with no attachment keys falls back to visible.
    {
        let mut animation = app.world_mut().get_mut::<Wc3Animation>(root).unwrap();
        animation
            .sequences
            .push(Sequence::new("Alternate", [3000, 4000]).unwrap());
        animation.play(1);
    }
    app.world_mut()
        .get_mut::<Wc3Animation>(weapon)
        .unwrap()
        .elapsed_ms = 123.0;
    app.update();
    assert_eq!(app.world().get::<Wc3Animation>(weapon).unwrap().sequence, 0);
    assert_eq!(
        app.world().get::<Wc3Animation>(weapon).unwrap().elapsed_ms,
        0.0
    );
    *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Hidden;
    app.update();
    assert!(!app.world().get::<Wc3Animation>(weapon).unwrap().playing);
    *app.world_mut().get_mut::<Visibility>(root).unwrap() = Visibility::Inherited;
    app.update();
    assert!(app.world().get::<Wc3Animation>(weapon).unwrap().playing);
    app.world_mut().entity_mut(root).despawn();
    let mut transforms = app.world_mut().query::<&Transform>();
    assert_eq!(transforms.iter(app.world()).count(), 0);
}

#[test]
fn delayed_and_nested_attachments_restart_in_parent_order() {
    let mut app = app();
    let leaf = app
        .world_mut()
        .resource_mut::<Assets<Wc3ModelAsset>>()
        .add(asset(child_source(), default()));
    let child = app
        .world()
        .resource::<Assets<Wc3ModelAsset>>()
        .reserve_handle();
    let source = parent_source();
    let resources = Wc3ModelResources::resolve(&source.model, |_| Some(child.clone()));
    let parent = app
        .world_mut()
        .resource_mut::<Assets<Wc3ModelAsset>>()
        .add(asset(source, resources));
    let root = app.world_mut().spawn(Wc3ModelInstance::new(parent)).id();
    app.update();
    let weapon = points(&app, root)[0].model.unwrap();
    seek(&mut app, root, 750.0);
    assert!(app.world().get::<Wc3Animation>(weapon).is_none());
    let mut source = child_source();
    let mut node = Node::new("Nested Ref", 1).unwrap();
    node.parent_id = 0;
    node.flags.set_attachment(true);
    let mut nested = Attachment::new(node, "leaf.mdl", 99).unwrap();
    nested.visibility = Some(
        Track::step(
            vec![
                ValueKeyframe {
                    frame: 0,
                    value: 0.0,
                },
                ValueKeyframe {
                    frame: 100,
                    value: 1.0,
                },
            ],
            None,
        )
        .unwrap(),
    );
    source.model.set_attachments(&[nested]);
    source.model.set_pivot_points(&[[0.0; 3], [0.0; 3]]);
    let resources = Wc3ModelResources::resolve(&source.model, |_| Some(leaf.clone()));
    app.world_mut()
        .resource_mut::<Assets<Wc3ModelAsset>>()
        .insert(&child, asset(source, resources))
        .unwrap();
    app.update();
    app.update();
    let nested = points(&app, weapon)[0].clone();
    let leaf_root = nested.model.unwrap();
    assert!(!app.world().get::<Wc3Animation>(weapon).unwrap().playing);
    assert!(!app.world().get::<Wc3Animation>(leaf_root).unwrap().playing);
    seek(&mut app, root, 1000.0);
    assert_eq!(
        app.world().get::<Wc3Animation>(weapon).unwrap().elapsed_ms,
        0.0
    );
    assert!(!app
        .world()
        .get::<InheritedVisibility>(leaf_root)
        .unwrap()
        .get());
    seek(&mut app, weapon, 200.0);
    assert!(app
        .world()
        .get::<InheritedVisibility>(leaf_root)
        .unwrap()
        .get());
    assert!(app.world().get::<Wc3Animation>(leaf_root).unwrap().playing);
    // A seek/restart on the outer model resets the child's first sequence
    // before the grandchild's visibility is evaluated in this frame.
    seek(&mut app, root, 0.0);
    assert_eq!(
        app.world().get::<Wc3Animation>(weapon).unwrap().elapsed_ms,
        0.0
    );
    assert!(!app
        .world()
        .get::<InheritedVisibility>(leaf_root)
        .unwrap()
        .get());
    assert!(!app.world().get::<Wc3Animation>(leaf_root).unwrap().playing);
    app.world_mut().entity_mut(root).despawn();
    assert!(app.world().get_entity(leaf_root).is_err());
}

#[test]
fn recursive_model_paths_do_not_expand_forever() {
    let mut app = app();
    let assets = app.world().resource::<Assets<Wc3ModelAsset>>();
    let a = assets.reserve_handle();
    let b = assets.reserve_handle();
    let source = || {
        Wc3Model::decode_mdl(
            r#"
            Version { FormatVersion 800, } Model "Cycle" {}
            Attachment "Nested Ref" { ObjectId 0, Path "cycle.mdl", }
        "#,
        )
        .unwrap()
    };
    for (handle, child) in [(&a, &b), (&b, &a)] {
        let source = source();
        let models = Wc3ModelResources::resolve(&source.model, |_| Some(child.clone()));
        app.world_mut()
            .resource_mut::<Assets<Wc3ModelAsset>>()
            .insert(handle, asset(source, models))
            .unwrap();
    }
    let root = app.world_mut().spawn(Wc3ModelInstance::new(a)).id();
    for _ in 0..10 {
        app.update();
    }
    let mut instances = app.world_mut().query::<&Wc3ModelInstance>();
    assert_eq!(instances.iter(app.world()).count(), 3);
    let child = points(&app, root)[0].model.unwrap();
    let blocked = points(&app, child)[0].model.unwrap();
    assert!(app.world().get::<Wc3Animation>(blocked).is_none());
    assert_eq!(
        *app.world().get::<Visibility>(blocked).unwrap(),
        Visibility::Hidden
    );
    app.world_mut().entity_mut(root).despawn();
    let mut transforms = app.world_mut().query::<&Transform>();
    assert_eq!(transforms.iter(app.world()).count(), 0);
}
