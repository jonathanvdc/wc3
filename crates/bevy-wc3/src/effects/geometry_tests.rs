use super::*;
use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::visibility::{RenderLayers, VisibilityPlugin, VisibleEntities};
use bevy::mesh::MeshPlugin;
use bevy::render::view::NoIndirectDrawing;
use std::any::TypeId;

#[test]
fn shared_quad_and_extraction_preserve_asset_identity_and_world_center_without_mesh_uniforms() {
    let mut meshes = Assets::<Mesh>::default();
    let first = effect_quad(&mut meshes);
    let second = effect_quad(&mut meshes);
    assert_eq!(first.id(), second.id());
    assert_eq!(meshes.len(), 1);
    let transform =
        GlobalTransform::from(Transform::from_xyz(3., -4., 5.).with_scale(Vec3::new(-2., 3., 4.)));
    let extracted =
        EffectGeometry::extract_component((&EffectGeometry(first.clone()), &transform)).unwrap();
    assert_eq!(extracted.mesh, first.id());
    assert_eq!(extracted.world_center, Vec3::new(3., -4., 5.));
}

#[test]
fn native_visibility_tracks_layers_parent_hidden_state_and_camera_choices_without_mesh3d() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        VisibilityPlugin,
        MeshPlugin,
    ));
    app.init_asset::<Mesh>();
    app.add_plugins(EffectGeometryPlugin);
    let root = app
        .world_mut()
        .spawn((Transform::default(), Visibility::Inherited))
        .id();
    let effect = app
        .world_mut()
        .spawn((
            EffectGeometry(Handle::default()),
            RenderLayers::layer(1),
            ChildOf(root),
        ))
        .id();
    let first = app
        .world_mut()
        .spawn((Camera3d::default(), RenderLayers::layer(0)))
        .id();
    let second = app
        .world_mut()
        .spawn((Camera3d::default(), RenderLayers::layer(1)))
        .id();
    app.update();
    let contains = |app: &App, view: Entity| {
        app.world()
            .get::<VisibleEntities>(view)
            .unwrap()
            .get(TypeId::of::<EffectGeometry>())
            .contains(&effect)
    };
    assert!(!contains(&app, first));
    assert!(contains(&app, second));
    assert!(!app.world().entity(effect).contains::<Mesh3d>());
    assert!(app.world().entity(effect).contains::<Aabb>());
    assert!(!app.world().entity(first).contains::<NoIndirectDrawing>());
    app.world_mut().entity_mut(root).insert(Visibility::Hidden);
    app.update();
    assert!(!contains(&app, second));
    app.world_mut().entity_mut(root).insert(Visibility::Visible);
    app.world_mut()
        .entity_mut(effect)
        .insert(RenderLayers::layer(0));
    app.update();
    assert!(contains(&app, first));
    assert!(!contains(&app, second));
}
