use bevy::asset::{AssetApp, AssetPlugin, AssetServer};
use bevy::image::{Image, ImageLoaderSettings};
use bevy::prelude::{default, App, Assets, Handle};
use bevy::tasks::{ComputeTaskPool, IoTaskPool, TaskPoolBuilder};
use bevy_wc3::{BlpImageLoader, UniversalModelAsset, UniversalModelLoader, UniversalPathResolver};
use std::borrow::Cow;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;
use wc3::model::formats::mdlx::{Document, V800};

fn wait_for(app: &mut App, mut ready: impl FnMut(&App) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        app.update();
        if ready(app) {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(ready(app), "assets did not load before the timeout");
}

fn init_task_pools() {
    IoTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    ComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    bevy::tasks::AsyncComputeTaskPool::get_or_init(|| {
        TaskPoolBuilder::new().num_threads(2).build()
    });
}

#[test]
fn asset_server_loads_mdx_and_mdl_files() {
    init_task_pools();
    let directory = std::env::temp_dir().join(format!(
        "bevy-wc3-assets-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("quad.mdx"),
        include_bytes!("fixtures/quad_model.mdx"),
    )
    .unwrap();
    fs::write(
        directory.join("quad.mdl"),
        include_str!("fixtures/quad_model.mdl"),
    )
    .unwrap();

    let mut app = App::new();
    app.add_plugins(AssetPlugin {
        file_path: directory.to_string_lossy().into_owned(),
        ..default()
    });
    app.init_asset::<Image>();
    app.init_asset::<UniversalModelAsset>();
    app.register_asset_loader(UniversalModelLoader::new(Arc::new(UniversalPathResolver)));
    app.init_asset_loader::<BlpImageLoader>();
    let handles: [Handle<UniversalModelAsset>; 2] =
        ["quad.mdx", "quad.mdl"].map(|path| app.world().resource::<AssetServer>().load(path));
    wait_for(&mut app, |app| {
        handles.iter().all(|handle| {
            app.world()
                .resource::<Assets<UniversalModelAsset>>()
                .contains(handle)
        })
    });
    let assets = app.world().resource::<Assets<UniversalModelAsset>>();
    let mdx = assets.get(&handles[0]).unwrap();
    let mdl = assets.get(&handles[1]).unwrap();
    assert_eq!(
        mdx.compiled
            .clips()
            .iter()
            .map(|(_, name)| name)
            .collect::<Vec<_>>(),
        mdl.compiled
            .clips()
            .iter()
            .map(|(_, name)| name)
            .collect::<Vec<_>>()
    );
    let bounds = |asset: &UniversalModelAsset| {
        asset
            .compiled
            .rest_bounds()
            .map(|bounds| (bounds.center, bounds.half_extents))
    };
    assert_eq!(bounds(mdx), bounds(mdl));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn asset_server_resolves_child_models_and_keeps_missing_slots() {
    init_task_pools();
    let directory =
        std::env::temp_dir().join(format!("bevy-wc3-child-assets-{}", std::process::id()));
    fs::create_dir_all(directory.join("units")).unwrap();
    let child = "Version { FormatVersion 800, } Model \"Child\" {}";
    let source = Document::<V800>::decode_mdl(child).unwrap();
    // Local MDX fallback wins over an MDL at the asset root.
    fs::write(
        directory.join("units/spark.mdx"),
        source.encode_mdx().unwrap(),
    )
    .unwrap();
    fs::write(directory.join("spark.mdl"), child).unwrap();
    // A real local MDL wins over its MDX sibling.
    fs::write(directory.join("units/weapon.mdl"), child).unwrap();
    fs::write(
        directory.join("units/weapon.mdx"),
        source.encode_mdx().unwrap(),
    )
    .unwrap();
    fs::write(directory.join("root.mdl"), child).unwrap();
    fs::write(
        directory.join("units/parent.mdl"),
        r#"
        Version { FormatVersion 800, } Model "Parent" {}
        Attachment "Empty" { ObjectId 0, }
        Attachment "Weapon" { ObjectId 1, Path "weapon.mdl", }
        Attachment "Root" { ObjectId 2, Path "root.mdl", }
        Attachment "Missing" { ObjectId 3, Path "missing.mdx", }
        ParticleEmitter "Spark" { ObjectId 4, EmitterUsesMdl, Path "spark.mdl", }
        ParticleEmitter "Image" { ObjectId 5, EmitterUsesTga, Path "spark.tga", }
    "#,
    )
    .unwrap();

    let mut app = App::new();
    app.add_plugins(AssetPlugin {
        file_path: directory.to_string_lossy().into_owned(),
        ..default()
    });
    app.init_asset::<Image>();
    app.init_asset::<UniversalModelAsset>();
    app.register_asset_loader(UniversalModelLoader::new(Arc::new(UniversalPathResolver)));
    app.init_asset_loader::<BlpImageLoader>();
    let handle: Handle<UniversalModelAsset> = app
        .world()
        .resource::<AssetServer>()
        .load("units/parent.mdl");
    wait_for(&mut app, |app| {
        app.world()
            .resource::<AssetServer>()
            .is_loaded_with_dependencies(&handle)
    });
    let assets = app.world().resource::<Assets<UniversalModelAsset>>();
    assert!(assets
        .get(&handle)
        .unwrap()
        .compiled
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.feature.is_some()
            && diagnostic.message.contains("could not be resolved")));
    let server = app.world().resource::<AssetServer>();
    for path in ["units/weapon.mdl", "root.mdl", "units/spark.mdx"] {
        let child = server
            .get_handle::<UniversalModelAsset>(path)
            .expect("selected child registered");
        assert!(assets.contains(&child), "child {path} loaded");
    }
    assert!(server
        .get_handle::<UniversalModelAsset>("spark.mdl")
        .is_none());
    assert!(server
        .get_handle::<UniversalModelAsset>("units/weapon.mdx")
        .is_none());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn blp0_file_loader_reads_base_sidecar_and_reports_missing_sidecar() {
    use bevy::asset::LoadState;
    use wc3::blp::{Blp0, Blp0Content, Blp0Header};

    init_task_pools();
    let directory = std::env::temp_dir().join(format!("bevy-wc3-blp0-{}", std::process::id()));
    fs::create_dir_all(directory.join("textures")).unwrap();
    let mut palette = Box::new([0; 1024]);
    palette[..4].copy_from_slice(&[30, 20, 10, 0]);
    let output = Blp0 {
        header: Blp0Header {
            width: 1,
            height: 1,
            alpha_bits: 8,
            extra: 4,
            has_mipmaps: 0,
        },
        content: Blp0Content::Indexed {
            palette: palette.into(),
        },
        mipmaps: std::array::from_fn(|level| (level == 0).then(|| Cow::Owned(vec![0, 160]))),
    }
    .write_files()
    .unwrap();
    fs::write(directory.join("textures/present.blp"), &output.root).unwrap();
    fs::write(
        directory.join("textures/present.b00"),
        output.mipmaps[0].as_ref().unwrap(),
    )
    .unwrap();
    fs::write(directory.join("textures/missing.blp"), &output.root).unwrap();
    let mut app = App::new();
    app.add_plugins(AssetPlugin {
        file_path: directory.to_string_lossy().into_owned(),
        ..default()
    });
    app.init_asset::<Image>()
        .init_asset_loader::<BlpImageLoader>();
    let server = app.world().resource::<AssetServer>();
    let present: Handle<Image> = server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.is_srgb = false;
        })
        .load("textures/present.blp");
    let missing: Handle<Image> = server.load("textures/missing.blp");
    wait_for(&mut app, |app| {
        app.world().resource::<Assets<Image>>().contains(&present)
            && matches!(
                app.world()
                    .resource::<AssetServer>()
                    .get_load_state(&missing),
                Some(LoadState::Failed(_))
            )
    });
    let images = app.world().resource::<Assets<Image>>();
    let image = images.get(&present).unwrap();
    assert_eq!(image.data.as_deref(), Some([10, 20, 30, 160].as_slice()));
    assert_eq!(
        image.texture_descriptor.format,
        bevy::render::render_resource::TextureFormat::Rgba8Unorm
    );
    let Some(LoadState::Failed(error)) = app
        .world()
        .resource::<AssetServer>()
        .get_load_state(&missing)
    else {
        panic!("expected missing sidecar failure")
    };
    assert!(error.to_string().contains("textures/missing.b00"));
    fs::remove_dir_all(directory).unwrap();
}
