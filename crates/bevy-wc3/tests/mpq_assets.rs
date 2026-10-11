use bevy::asset::{io::AssetSourceBuilder, AssetApp, AssetPlugin};
use bevy::image::ImageLoader;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool, TaskPoolBuilder};
use bevy_mpq::{MpqAssetReader, OverlayAssetReader, OverlayMount};
use bevy_wc3::{Wc3ModelAsset, Wc3ModelLoader};
use std::borrow::Cow;
use std::io::Cursor;
use std::thread::sleep;
use std::time::{Duration, Instant};
use wc3::mpq::{Archive, ArchiveWriter, FileOptions, SeekSource, WriteOptions};

fn mount(entries: &[(&str, &[u8])]) -> MpqAssetReader<SeekSource<Cursor<Vec<u8>>>> {
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
        Archive::open(Cursor::new(writer.finish().unwrap().into_inner()))
            .unwrap()
            .into_shared(),
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
                OverlayMount::mpq(map.clone()),
                OverlayMount::mpq(base.clone()),
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

#[test]
fn blp0_base_mipmap_loads_from_named_archive_source() {
    use bevy_wc3::BlpImageLoader;
    use wc3::blp::{Blp0, Blp0Content, Blp0Header};

    IoTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    ComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    AsyncComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    let mut palette = Box::new([0; 1024]);
    palette[..4].copy_from_slice(&[30, 20, 10, 0]);
    let texture = Blp0 {
        header: Blp0Header {
            width: 1,
            height: 1,
            alpha_bits: 8,
            extra: 4,
            has_mipmaps: 1,
        },
        content: Blp0Content::Indexed {
            palette: palette.into(),
        },
        mipmaps: std::array::from_fn(|level| (level == 0).then(|| Cow::Owned(vec![0, 160]))),
    }
    .write_files()
    .unwrap();
    let root = mount(&[("Textures/Beta.blp", &texture.root)]);
    let sidecar = mount(&[("Textures/Beta.b00", texture.mipmaps[0].as_ref().unwrap())]);
    let mut app = App::new();
    app.register_asset_source(
        "beta",
        AssetSourceBuilder::new(move || {
            Box::new(OverlayAssetReader::new(vec![
                OverlayMount::mpq(root.clone()),
                OverlayMount::mpq(sidecar.clone()),
            ]))
        }),
    );
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>()
        .init_asset_loader::<BlpImageLoader>();
    let handle: Handle<Image> = app
        .world()
        .resource::<AssetServer>()
        .load("beta://Textures/Beta.blp");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        app.update();
        if app.world().resource::<Assets<Image>>().contains(&handle) {
            break;
        }
        assert!(Instant::now() < deadline, "BLP0 sidecar failed to load");
        sleep(Duration::from_millis(5));
    }
    let images = app.world().resource::<Assets<Image>>();
    let image = images.get(&handle).unwrap();
    assert_eq!(image.data.as_deref(), Some([10, 20, 30, 160].as_slice()));
    assert_eq!(
        image.texture_descriptor.format,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb
    );
}
