//! cargo run -p bevy-wc3 --example mpq -- units/human/footman/footman.mdx map.w3x base.mpq
//! Archives are searched in argument order. The application chooses the namespace
//! and precedence; bevy-wc3 only resolves dependencies within that namespace.
use bevy::asset::{
    io::{AssetSourceBuilder, ErasedAssetReader},
    AssetApp,
};
use bevy::prelude::*;
use bevy_mpq::{MpqAssetReader, OverlayAssetReader};
use bevy_wc3::{Wc3BevyPlugin, Wc3ModelAsset, Wc3ModelInstance};
use std::env::args;
use std::fs::File;
use wc3::mpq::{Archive, ReadOptions};

#[derive(Resource)]
struct ModelPath(String);

fn main() {
    let mut arguments = args().skip(1);
    let model = arguments
        .next()
        .expect("pass a model path within the archives");
    let readers: Vec<_> = arguments
        .map(|path| {
            let file = File::open(&path).expect("open archive");
            let archive = Archive::with_options(
                file,
                ReadOptions {
                    max_file_size: 128 << 20,
                    ..default()
                },
            )
            .expect("index archive");
            MpqAssetReader::new(archive, path)
        })
        .collect();
    assert!(!readers.is_empty(), "pass at least one archive path");
    let mut app = App::new();
    app.register_asset_source(
        "warcraft",
        AssetSourceBuilder::new(move || {
            Box::new(OverlayAssetReader::new(
                readers
                    .iter()
                    .map(|reader| Box::new(reader.clone()) as Box<dyn ErasedAssetReader>)
                    .collect(),
            ))
        }),
    );
    app.add_plugins((DefaultPlugins, Wc3BevyPlugin))
        .insert_resource(ModelPath(format!("warcraft://{model}")))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, path: Res<ModelPath>) {
    let model: Handle<Wc3ModelAsset> = assets.load(path.0.clone());
    commands.spawn((Wc3ModelInstance::new(model), Transform::default()));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(300.0, -500.0, 250.0).looking_at(Vec3::new(0.0, 0.0, 60.0), Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 5_000.0,
            ..default()
        },
        Transform::from_xyz(100.0, -100.0, 200.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}
