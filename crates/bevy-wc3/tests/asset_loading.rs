use bevy::asset::{AssetApp, AssetPlugin, AssetServer};
use bevy::image::Image;
use bevy::prelude::{default, App, Assets, Handle};
use bevy::tasks::{ComputeTaskPool, IoTaskPool, TaskPoolBuilder};
use bevy_wc3::{BlpImageLoader, Wc3ModelAsset, Wc3ModelLoader};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use wc3::model::mdl::Write as _;
use wc3::model::mdx::Write as _;

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
        include_bytes!("../../wc3/tests/fixtures/mdl/quad_model.mdx"),
    )
    .unwrap();
    fs::write(
        directory.join("quad.mdl"),
        include_str!("../../wc3/tests/fixtures/mdl/quad_model.mdl"),
    )
    .unwrap();

    let mut app = App::new();
    app.add_plugins(AssetPlugin {
        file_path: directory.to_string_lossy().into_owned(),
        ..default()
    });
    app.init_asset::<Image>();
    app.init_asset::<Wc3ModelAsset>();
    app.init_asset_loader::<Wc3ModelLoader>();
    app.init_asset_loader::<BlpImageLoader>();
    let handles: [Handle<Wc3ModelAsset>; 2] =
        ["quad.mdx", "quad.mdl"].map(|path| app.world().resource::<AssetServer>().load(path));
    wait_for(&mut app, |app| {
        handles.iter().all(|handle| {
            app.world()
                .resource::<Assets<Wc3ModelAsset>>()
                .contains(handle)
        })
    });
    let assets = app.world().resource::<Assets<Wc3ModelAsset>>();
    assert_eq!(
        assets
            .get(&handles[0])
            .unwrap()
            .source()
            .model
            .encode_mdl()
            .unwrap(),
        assets
            .get(&handles[1])
            .unwrap()
            .source()
            .model
            .encode_mdl()
            .unwrap()
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn asset_server_resolves_child_models_and_keeps_missing_slots() {
    init_task_pools();
    let directory =
        std::env::temp_dir().join(format!("bevy-wc3-child-assets-{}", std::process::id()));
    fs::create_dir_all(directory.join("units")).unwrap();
    let child = "Version { FormatVersion 800, } Model \"Child\" {}";
    let source = bevy_wc3::Wc3Model::decode_mdl(child).unwrap();
    // Local MDX fallback wins over an MDL at the asset root.
    fs::write(
        directory.join("units/spark.mdx"),
        source.model.encode_mdx().unwrap(),
    )
    .unwrap();
    fs::write(directory.join("spark.mdl"), child).unwrap();
    // A real local MDL wins over its MDX sibling.
    fs::write(directory.join("units/weapon.mdl"), child).unwrap();
    fs::write(
        directory.join("units/weapon.mdx"),
        source.model.encode_mdx().unwrap(),
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
    app.init_asset::<Wc3ModelAsset>();
    app.init_asset_loader::<Wc3ModelLoader>();
    app.init_asset_loader::<BlpImageLoader>();
    let handle: Handle<Wc3ModelAsset> = app
        .world()
        .resource::<AssetServer>()
        .load("units/parent.mdl");
    wait_for(&mut app, |app| {
        app.world()
            .resource::<AssetServer>()
            .is_loaded_with_dependencies(&handle)
    });
    let assets = app.world().resource::<Assets<Wc3ModelAsset>>();
    let resources = assets.get(&handle).unwrap().model_resources();
    assert!(resources.attachment(0).is_none());
    assert!(resources.attachment(3).is_none());
    assert!(resources.particle(1).is_none());
    for (child, path) in [
        (resources.attachment(1).unwrap(), "units/weapon.mdl"),
        (resources.attachment(2).unwrap(), "root.mdl"),
        (resources.particle(0).unwrap(), "units/spark.mdx"),
    ] {
        assert_eq!(child.path().unwrap().path(), Path::new(path));
        assert!(assets.contains(&child));
    }
    fs::remove_dir_all(directory).unwrap();
}
