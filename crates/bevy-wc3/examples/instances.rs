//! Put an MDX and its textures under `assets/`, then run:
//! `cargo run -p bevy-wc3 --example instances -- units/footman.mdx`.
use bevy::prelude::*;
use bevy_wc3::{Wc3BevyPlugin, Wc3ModelInstance};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Wc3BevyPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let path = std::env::args()
        .nth(1)
        .expect("pass an MDX path relative to assets/");
    let model = assets.load(path);
    for x in [-150.0, 150.0] {
        commands.spawn((
            Wc3ModelInstance::new(model.clone()),
            Transform::from_xyz(x, 0.0, 0.0),
        ));
    }
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, -650.0, 350.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(1.0, -1.0, 2.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}
