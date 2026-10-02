use super::*;
#[test]
fn texture_sampler_uses_wrap_flags_per_axis() {
    let mut flags = TextureFlags::default();
    flags.set_wrap_width(true);
    let ImageSampler::Descriptor(sampler) = texture_sampler(flags) else {
        panic!("expected a texture sampler descriptor");
    };
    assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
    assert_eq!(sampler.address_mode_v, ImageAddressMode::ClampToEdge);

    flags.set_wrap_width(false);
    flags.set_wrap_height(true);
    let ImageSampler::Descriptor(sampler) = texture_sampler(flags) else {
        panic!("expected a texture sampler descriptor");
    };
    assert_eq!(sampler.address_mode_u, ImageAddressMode::ClampToEdge);
    assert_eq!(sampler.address_mode_v, ImageAddressMode::Repeat);
}

#[test]
fn replaceable_bitmap_has_no_guessed_path() {
    let mut bitmap = Texture::new("").unwrap();
    bitmap.replaceable_id = 31;
    let mut requested = Vec::new();
    let binding = resolve_texture(&bitmap, &mut |path| {
        requested.push(path.to_owned());
        None
    });
    assert!(requested.is_empty());
    assert_eq!(binding.replaceable_id, 31);
}

#[test]
fn bitmap_keeps_replaceable_id_and_only_resolves_explicit_path() {
    let mut bitmap = Texture::new("").unwrap();
    bitmap.replaceable_id = 11;
    let mut requested = Vec::new();
    let binding = resolve_texture(&bitmap, &mut |path| {
        requested.push(path.to_owned());
        None
    });
    assert!(requested.is_empty());
    assert_eq!(binding.replaceable_id, 11);

    bitmap.path.set_text("Custom/Cliff.blp").unwrap();
    resolve_texture(&bitmap, &mut |path| {
        requested.push(path.to_owned());
        None
    });
    assert_eq!(requested, ["Custom/Cliff.blp"]);
}

#[test]
fn file_loader_resolves_tif_references_and_preserves_location_precedence() {
    use bevy::asset::{AssetApp, AssetPlugin, AssetServer};
    use bevy::image::ImageLoader;
    use bevy::tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool, TaskPoolBuilder};
    use std::fs;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    IoTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    ComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    AsyncComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    let directory =
        std::env::temp_dir().join(format!("wc3-texture-fallback-{}", std::process::id()));
    fs::create_dir_all(directory.join("units")).unwrap();
    // A complete one-pixel, uncompressed, 24-bit TGA.
    let tga = [
        0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 24, 32, 0, 0, 255,
    ];
    fs::write(directory.join("units/body.tga"), tga).unwrap();
    fs::write(directory.join("body.tif"), b"root literal must not win").unwrap();
    fs::write(directory.join("root.tga"), tga).unwrap();
    fs::write(directory.join("units/exact.tga"), tga).unwrap();
    fs::write(directory.join("units/exact.blp"), b"alternate must not win").unwrap();
    fs::write(
        directory.join("units/model.mdl"),
        r#"
        Version { FormatVersion 800, } Model "Textures" {}
        Textures 3 {
            Bitmap { Image "body.tif", }
            Bitmap { Image "root.TIF", }
            Bitmap { Image "exact.tga", }
        }
    "#,
    )
    .unwrap();
    let mut app = App::new();
    app.add_plugins(AssetPlugin {
        file_path: directory.to_string_lossy().into_owned(),
        ..default()
    });
    app.init_asset::<Image>();
    app.register_asset_loader(ImageLoader::new(default()));
    app.init_asset::<Wc3ModelAsset>();
    app.init_asset_loader::<Wc3ModelLoader>();
    let handle: Handle<Wc3ModelAsset> = app
        .world()
        .resource::<AssetServer>()
        .load("units/model.mdl");
    let deadline = Instant::now() + Duration::from_secs(5);
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
            "texture dependencies did not load"
        );
        sleep(Duration::from_millis(5));
    }
    let models = app.world().resource::<Assets<Wc3ModelAsset>>();
    let model = models.get(&handle).unwrap();
    for (bitmap, expected) in
        model
            .textures
            .bitmaps
            .iter()
            .zip(["units/body.tga", "root.tga", "units/exact.tga"])
    {
        let image = bitmap.default.as_ref().unwrap();
        assert_eq!(image.path().unwrap().path(), Path::new(expected));
        let images = app.world().resource::<Assets<Image>>();
        assert_eq!(images.get(image).unwrap().width(), 1);
    }
    fs::remove_dir_all(directory).unwrap();
}
