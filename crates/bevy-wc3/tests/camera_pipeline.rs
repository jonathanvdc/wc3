use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::visibility::VisibilityPlugin;
use bevy::image::ImagePlugin;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy::render::view::NoIndirectDrawing;
use bevy::shader::Shader;
use bevy_wc3::Wc3BevyPlugin;

#[test]
fn plugin_preserves_application_camera_preprocessing_choices() {
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
