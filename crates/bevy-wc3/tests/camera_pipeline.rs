use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::visibility::{RenderLayers, VisibilityClass, VisibilityPlugin, VisibleEntities};
use bevy::ecs::world::CommandQueue;
use bevy::image::ImagePlugin;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy::render::view::NoIndirectDrawing;
use bevy::shader::Shader;
use bevy_wc3::{compile_model, spawn_model, ModelMaterial, SpriteEmitter, Wc3BevyPlugin};
use wc3::model::{conversion::ImportOptions, io, profiles::Universal};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        VisibilityPlugin,
        MeshPlugin,
        ImagePlugin::default(),
    ));
    app.init_asset::<Shader>();
    app.add_plugins(Wc3BevyPlugin);
    app
}

#[test]
fn plugin_preserves_application_camera_preprocessing_choices() {
    let mut app = app();
    let ordinary = app.world_mut().spawn(Camera3d::default()).id();
    let direct = app
        .world_mut()
        .spawn((Camera3d::default(), NoIndirectDrawing))
        .id();

    app.update();
    app.update();

    assert!(!app.world().entity(ordinary).contains::<NoIndirectDrawing>());
    assert!(app.world().entity(direct).contains::<NoIndirectDrawing>());
}

#[test]
fn procedural_emitters_join_native_per_camera_visibility_and_render_layers() {
    let mut app = app();
    particle_model(&mut app);
    let first_view = app
        .world_mut()
        .spawn((Camera3d::default(), RenderLayers::layer(0)))
        .id();
    let second_view = app
        .world_mut()
        .spawn((Camera3d::default(), RenderLayers::layer(1)))
        .id();
    app.update();
    let emitters: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<SpriteEmitter>>()
        .iter(app.world())
        .collect();
    assert!(!emitters.is_empty());
    for &emitter in &emitters {
        assert!(!app.world().entity(emitter).contains::<Mesh3d>());
        assert!(visible(&app, first_view, emitter));
        assert!(!visible(&app, second_view, emitter));
        app.world_mut()
            .entity_mut(emitter)
            .insert(RenderLayers::layer(1));
    }
    app.update();
    for emitter in emitters {
        assert!(!visible(&app, first_view, emitter));
        assert!(visible(&app, second_view, emitter));
    }
}

fn visible(app: &App, view: Entity, entity: Entity) -> bool {
    let visible = app.world().get::<VisibleEntities>(view).unwrap();
    app.world()
        .get::<VisibilityClass>(entity)
        .is_some_and(|classes| {
            classes
                .iter()
                .any(|class| visible.get(*class).contains(&entity))
        })
}

#[test]
fn ribbon_material_passes_join_native_per_view_visibility() {
    use bevy_wc3::RibbonEmitter;
    let mut app = app();
    fixture_model(&mut app, include_str!("fixtures/ribbon_capture.mdl"));
    let view = app.world_mut().spawn(Camera3d::default()).id();
    app.update();
    let passes: Vec<_> = app
        .world_mut()
        .query_filtered::<(Entity, &ChildOf), With<VisibilityClass>>()
        .iter(app.world())
        .filter_map(|(entity, parent)| {
            app.world()
                .get::<RibbonEmitter>(parent.parent())
                .map(|_| entity)
        })
        .collect();
    assert!(!passes.is_empty());
    for pass in passes {
        assert!(visible(&app, view, pass));
    }
}

fn particle_model(app: &mut App) -> Entity {
    fixture_model(app, include_str!("fixtures/particle_capture.mdl"))
}

fn fixture_model(app: &mut App, mdl: &str) -> Entity {
    let source = io::mdlx::decode_mdl(mdl, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let compiled = compile_model(&source, |_| None).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let prepared = compiled.prepare(
        &mut meshes,
        &mut Assets::<SkinnedMeshInverseBindposes>::default(),
    );
    let mut queue = CommandQueue::default();
    let root = spawn_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut Assets::<ModelMaterial>::default(),
        &prepared,
        Transform::default(),
    );
    queue.apply(app.world_mut());
    root
}

fn model_emitters(app: &mut App, root: Entity) -> Vec<Entity> {
    let emitters: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<SpriteEmitter>>()
        .iter(app.world())
        .collect();
    emitters
        .into_iter()
        .filter(|&emitter| {
            let mut entity = emitter;
            while let Some(parent) = app.world().get::<ChildOf>(entity) {
                entity = parent.parent();
                if entity == root {
                    return true;
                }
            }
            false
        })
        .collect()
}

#[test]
fn model_root_layers_follow_changes_and_preserve_explicit_draws_and_application_children() {
    use bevy::camera::visibility::NoFrustumCulling;
    let mut app = app();
    let root = particle_model(&mut app);
    app.world_mut()
        .entity_mut(root)
        .insert(RenderLayers::layer(1));
    let child = app
        .world_mut()
        .spawn((Mesh3d(Handle::default()), NoFrustumCulling, ChildOf(root)))
        .id();
    let views: Vec<_> = (0..3)
        .map(|layer| {
            app.world_mut()
                .spawn((Camera3d::default(), RenderLayers::layer(layer)))
                .id()
        })
        .collect();
    app.update();
    let emitters = model_emitters(&mut app, root);
    assert!(emitters.len() >= 2);
    for &emitter in &emitters {
        assert!(visible(&app, views[1], emitter));
        assert!(!visible(&app, views[0], emitter));
    }
    assert!(app.world().get::<RenderLayers>(child).is_none());
    assert!(visible(&app, views[0], child));
    // Even writing the currently inherited value creates an explicit override.
    app.world_mut()
        .entity_mut(emitters[0])
        .insert(RenderLayers::layer(1));
    app.world_mut()
        .entity_mut(root)
        .insert(RenderLayers::layer(2));
    app.update();
    app.update();
    assert!(visible(&app, views[1], emitters[0]));
    for &emitter in &emitters[1..] {
        assert!(visible(&app, views[2], emitter));
    }
    app.world_mut().entity_mut(root).remove::<RenderLayers>();
    app.update();
    assert!(visible(&app, views[1], emitters[0]));
    for &emitter in &emitters[1..] {
        assert!(visible(&app, views[0], emitter));
    }
    app.world_mut()
        .entity_mut(emitters[0])
        .remove::<RenderLayers>();
    app.update();
    assert!(visible(&app, views[0], emitters[0]));
    app.world_mut()
        .entity_mut(root)
        .insert(RenderLayers::layer(2));
    app.update();
    assert!(emitters
        .iter()
        .all(|&emitter| visible(&app, views[2], emitter)));
    assert!(app.world().get::<RenderLayers>(child).is_none());
    assert!(visible(&app, views[0], child));
}

#[test]
fn nested_model_roots_select_the_nearest_explicit_layers() {
    let mut app = app();
    let parent = particle_model(&mut app);
    let child = particle_model(&mut app);
    app.world_mut()
        .entity_mut(parent)
        .insert(RenderLayers::layer(1));
    app.world_mut().entity_mut(child).insert(ChildOf(parent));
    app.update();
    let emitters = model_emitters(&mut app, child);
    assert!(!emitters.is_empty());
    for &emitter in &emitters {
        assert_eq!(
            app.world().get::<RenderLayers>(emitter),
            Some(&RenderLayers::layer(1))
        );
    }
    app.world_mut()
        .entity_mut(child)
        .insert(RenderLayers::layer(2));
    app.update();
    for &emitter in &emitters {
        assert_eq!(
            app.world().get::<RenderLayers>(emitter),
            Some(&RenderLayers::layer(2))
        );
    }
    app.world_mut().entity_mut(child).remove::<RenderLayers>();
    app.update();
    for emitter in emitters {
        assert_eq!(
            app.world().get::<RenderLayers>(emitter),
            Some(&RenderLayers::layer(1))
        );
    }
}

#[test]
fn animated_ribbon_fixture_builds_connected_live_segments_at_sixty_hertz() {
    use bevy::time::TimeUpdateStrategy;
    use bevy_wc3::{ModelRig, Playback, RibbonEmitter};
    use std::time::Duration;
    let mut app = app();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1. / 60.,
    )));
    let root = fixture_model(&mut app, include_str!("fixtures/ribbon_capture.mdl"));
    let clip = app.world().get::<ModelRig>(root).unwrap().model().clips()[0].0;
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .play_immediately(clip);
    app.world_mut().spawn(Camera3d::default());
    for _ in 0..61 {
        app.update();
    }
    let stats: Vec<_> = app
        .world_mut()
        .query::<&RibbonEmitter>()
        .iter(app.world())
        .map(|emitter| (emitter.sections, emitter.segments, emitter.omitted_sections))
        .collect();
    assert_eq!(stats.len(), 1);
    assert!(
        stats[0].0 >= 18,
        "expected the authored 20 Hz live sections: {stats:?}"
    );
    assert!(
        stats[0].1 >= 17,
        "expected connected ribbon geometry: {stats:?}"
    );
    assert_eq!(stats[0].2, 0);
    assert!((app.world().get::<Playback>(root).unwrap().shared_time - 1.).abs() < 0.02);
}
