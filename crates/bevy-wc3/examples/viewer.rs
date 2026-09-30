//! Run with `cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx`.
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_wc3::{Wc3Animation, Wc3BevyPlugin, Wc3ModelAsset, Wc3ModelInstance};
use std::path::PathBuf;

#[derive(Resource)]
struct Source {
    path: PathBuf,
    handle: Handle<Wc3ModelAsset>,
}

fn main() {
    let path = PathBuf::from(std::env::args().nth(1).expect("pass an MDX path"))
        .canonicalize()
        .expect("find MDX file");
    let directory = path.parent().expect("MDX parent directory");
    let filename = path
        .file_name()
        .expect("MDX filename")
        .to_string_lossy()
        .into_owned();
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: directory.to_string_lossy().into_owned(),
                ..default()
            }),
            Wc3BevyPlugin,
        ))
        .add_systems(
            Startup,
            move |mut commands: Commands, assets: Res<AssetServer>| {
                let handle = assets.load(filename.clone());
                commands.spawn(Wc3ModelInstance::new(handle.clone()));
                commands.insert_resource(Source {
                    path: path.clone(),
                    handle,
                });
                commands.spawn((
                    DirectionalLight {
                        illuminance: 20_000.0,
                        ..default()
                    },
                    Transform::from_xyz(1.0, -1.0, 2.0).looking_at(Vec3::ZERO, Vec3::Z),
                ));
            },
        )
        .add_systems(Update, (frame_model, controls))
        .run();
}

fn frame_model(
    mut commands: Commands,
    source: Res<Source>,
    models: Res<Assets<Wc3ModelAsset>>,
    cameras: Query<Entity, With<Camera3d>>,
) {
    if !cameras.is_empty() {
        return;
    }
    let Some(asset) = models.get(&source.handle) else {
        return;
    };
    let model = asset.source();
    let info = model.model.model_info();
    let center = info
        .as_ref()
        .map(|info| {
            (Vec3::from_array(info.minimum_extent) + Vec3::from_array(info.maximum_extent)) * 0.5
        })
        .unwrap_or(Vec3::ZERO);
    let radius = info
        .map(|info| info.bounds_radius)
        .unwrap_or(100.0)
        .max(10.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(center + Vec3::new(radius * 1.6, -radius * 2.2, radius * 0.9))
            .looking_at(center, Vec3::Z),
    ));
    info!(
        "loaded {:?}, source version {}; press Space to change sequence",
        source.path, model.source_version
    );
}

fn controls(keys: Res<ButtonInput<KeyCode>>, mut animations: Query<&mut Wc3Animation>) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    for mut animation in &mut animations {
        let next = (animation.sequence + 1) % animation.sequences().len().max(1);
        animation.play(next);
        info!("sequence {}", next);
    }
}
