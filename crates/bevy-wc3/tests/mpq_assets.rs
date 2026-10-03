use bevy::asset::{io::AssetSourceBuilder, AssetApp, AssetPlugin};
use bevy::image::ImageLoader;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool, TaskPoolBuilder};
use bevy_mpq::{MpqAssetReader, OverlayAssetReader};
use bevy_wc3::{Wc3ModelAsset, Wc3ModelLoader};
use std::io::Cursor;
use std::thread::sleep;
use std::time::{Duration, Instant};
use wc3::mpq::{Archive, ArchiveWriter, FileOptions, WriteOptions};

fn mount(entries: &[(&str, &[u8])]) -> MpqAssetReader<Cursor<Vec<u8>>> {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            listfile: false,
            ..default()
        },
    )
    .unwrap();
    for &(name, bytes) in entries {
        writer
            .add_file(
                name,
                bytes.len() as u32,
                &mut &*bytes,
                FileOptions::default(),
            )
            .unwrap();
    }
    MpqAssetReader::new(
        Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap(),
        "fixture",
    )
}

#[test]
fn named_source_loads_recursive_models_and_images_across_archives() {
    IoTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    ComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    AsyncComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    let parent = br#"Version { FormatVersion 800, } Model "Parent" {}
        Attachment "mount" { ObjectId 0, AttachmentID 0, Path "child.mdl", }
        ParticleEmitter "particles" { ObjectId 1, EmitterUsesMdl, Path "child.mdl", }
        PivotPoints 2 { { 0, 0, 0 }, { 0, 0, 0 }, }
    "#;
    let child = br#"Version { FormatVersion 800, } Model "Child" {}
        Textures 1 { Bitmap { Image "Textures\Body.tif", } }
        Attachment "mount" { ObjectId 0, AttachmentID 0, Path "leaf.mdl", }
        PivotPoints 1 { { 0, 0, 0 }, }
    "#;
    let leaf = br#"Version { FormatVersion 800, } Model "Leaf" {}"#;
    let tga = [
        0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 24, 32, 0, 0, 255,
    ];
    let map = mount(&[
        ("units/parent.mdl", parent),
        ("units/Textures/Body.tga", &tga),
    ]);
    let base = mount(&[
        ("units/child.mdl", child),
        ("units/leaf.mdl", leaf),
        ("Textures/Body.tga", b"root should not win"),
    ]);
    let mut app = App::new();
    app.register_asset_source(
        "map",
        AssetSourceBuilder::new(move || {
            Box::new(OverlayAssetReader::new(vec![
                Box::new(map.clone()),
                Box::new(base.clone()),
            ]))
        }),
    );
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>()
        .register_asset_loader(ImageLoader::new(default()));
    app.init_asset::<Wc3ModelAsset>()
        .init_asset_loader::<Wc3ModelLoader>();
    let handle: Handle<Wc3ModelAsset> = app
        .world()
        .resource::<AssetServer>()
        .load("map://units/parent.mdl");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        app.update();
        if app
            .world()
            .resource::<AssetServer>()
            .is_loaded_with_dependencies(&handle)
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "archive dependencies failed to load"
        );
        sleep(Duration::from_millis(5));
    }
    let models = app.world().resource::<Assets<Wc3ModelAsset>>();
    let particle = models
        .get(&handle)
        .unwrap()
        .model_resources()
        .particle(0)
        .unwrap();
    assert_eq!(
        particle.path().unwrap().to_string(),
        "map://units/child.mdl"
    );
    let child = models
        .get(&handle)
        .unwrap()
        .model_resources()
        .attachment(0)
        .unwrap();
    assert_eq!(child.path().unwrap().to_string(), "map://units/child.mdl");
    let leaf = models
        .get(&child)
        .unwrap()
        .model_resources()
        .attachment(0)
        .unwrap();
    assert_eq!(leaf.path().unwrap().to_string(), "map://units/leaf.mdl");
    assert!(models.get(&leaf).is_some());
    assert_eq!(app.world().resource::<Assets<Image>>().len(), 1);
}
