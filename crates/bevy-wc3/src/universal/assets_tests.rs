use super::*;
use crate::BlpImageLoader;
use bevy::asset::{AssetApp, AssetPlugin, LoadState};
use bevy::image::ImageLoader;
use bevy::tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool, TaskPoolBuilder};
use std::fs;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn app(directory: &Path) -> App {
    IoTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    ComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    AsyncComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    let mut app = App::new();
    app.add_plugins(AssetPlugin {
        file_path: directory.to_string_lossy().into_owned(),
        ..default()
    });
    app.init_asset::<Image>();
    app.register_asset_loader(ImageLoader::new(default()));
    app.init_asset_loader::<BlpImageLoader>();
    app.init_asset::<UniversalModelAsset>();
    app.init_asset_loader::<UniversalModelLoader>();
    app
}
fn directory(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("universal-{name}-{}", std::process::id()))
}
fn wait(app: &mut App, handle: &Handle<UniversalModelAsset>) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        app.update();
        let server = app.world().resource::<AssetServer>();
        if let LoadState::Failed(error) = server.load_state(handle) {
            return Err(error.to_string());
        }
        if server.is_loaded_with_dependencies(handle) {
            return Ok(());
        }
        assert!(Instant::now() < deadline, "asset loading timed out");
        sleep(Duration::from_millis(5));
    }
}
#[test]
fn path_provider_preserves_sources_and_does_not_guess_archive_id_names() {
    let resolver = UniversalPathResolver;
    let owner = AssetPath::parse("casc://units/unit.m2");
    let paths = resolver.candidates(
        &ModelResource::Path(Cow::Borrowed(b"textures\\body.blp")),
        &owner,
    );
    assert_eq!(
        paths.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["casc://units/textures/body.blp", "casc://textures/body.blp"]
    );
    assert_eq!(
        resolver.candidates(&ModelResource::Root, &owner),
        [owner.clone_owned()]
    );
    let relative = ModelResource::Relative {
        owner: Box::new(ModelResource::Root),
        suffix: Cow::Borrowed("00.skin"),
    };
    assert_eq!(
        resolver
            .candidates(&relative, &owner)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["casc://units/unit00.skin"]
    );
    assert!(resolver
        .candidates(&ModelResource::FileDataId(42), &owner)
        .is_empty());
    assert!(resolver
        .candidates(&ModelResource::Path(Cow::Borrowed(b"../outside")), &owner)
        .is_empty());
}
#[cfg(unix)]
#[test]
fn raw_path_and_relative_stem_keep_non_utf8_bytes() {
    use std::os::unix::ffi::OsStrExt as _;
    let owner = AssetPath::parse("casc://units/unit.m2");
    let relative = ModelResource::Relative {
        owner: Box::new(ModelResource::Path(Cow::Borrowed(b"raw/\xff.m2"))),
        suffix: Cow::Borrowed("00.skin"),
    };
    let paths = UniversalPathResolver.candidates(&relative, &owner);
    assert_eq!(
        paths[0].path().as_os_str().as_bytes(),
        b"units/raw/\xff00.skin"
    );
    assert_eq!(paths[1].path().as_os_str().as_bytes(), b"raw/\xff00.skin");
    assert_eq!(paths[0].source(), owner.source());
}

#[test]
fn native_m2_missing_skin_fails_with_resource_identity() {
    let directory = directory("missing-skin");
    fs::create_dir_all(&directory).unwrap();
    let mut native = m2::Document::<m2::Flat<m2::V264>>::new();
    native.core_mut().skins = 1;
    fs::write(directory.join("unit.m2"), native.encode_m2().unwrap()).unwrap();
    let mut app = app(&directory);
    let handle = app.world().resource::<AssetServer>().load("unit.m2");
    let error = wait(&mut app, &handle).unwrap_err();
    assert!(error.contains("required model companion"), "{error}");
    assert!(error.contains("unit00.skin"), "{error}");
    fs::remove_dir_all(directory).unwrap();
}
#[test]
fn semantic_mdl_loader_preserves_same_path_sampler_variants() {
    let directory = directory("image-variants");
    fs::create_dir_all(&directory).unwrap();
    let tga = [
        0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 24, 32, 0, 0, 255,
    ];
    fs::write(directory.join("body.tga"), tga).unwrap();
    fs::write(
        directory.join("unit.mdl"),
        r#"
        Version { FormatVersion 800, } Model "variants" {}
        Textures 2 {
            Bitmap { Image "body.tga", }
            Bitmap { Image "body.tga", WrapWidth, }
        }
    "#,
    )
    .unwrap();
    let mut app = app(&directory);
    let handle = app.world().resource::<AssetServer>().load("unit.mdl");
    wait(&mut app, &handle).unwrap();
    let models = app.world().resource::<Assets<UniversalModelAsset>>();
    assert!(models.get(&handle).unwrap().unresolved_images.is_empty());
    let images = app.world().resource::<Assets<Image>>();
    let samplers: Vec<_> = images
        .iter()
        .map(|(_, image)| {
            let ImageSampler::Descriptor(sampler) = &image.sampler else {
                panic!("sampler was discarded");
            };
            sampler.address_mode_u
        })
        .collect();
    assert_eq!(samplers.len(), 2);
    assert!(samplers.contains(&ImageAddressMode::Repeat));
    assert!(samplers.contains(&ImageAddressMode::ClampToEdge));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn native_hd_normal_and_orm_requests_keep_linear_gpu_image_formats() {
    use super::super::textures::TextureColorSpace;
    let directory = directory("role-variants");
    fs::create_dir_all(&directory).unwrap();
    let tga = [
        0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 24, 32, 128, 128, 255,
    ];
    fs::write(directory.join("shared.tga"), tga).unwrap();
    fs::write(
        directory.join("unit.mdl"),
        r#"
        Version { FormatVersion 1100, } Model "roles" {}
        Textures 1 { Bitmap { Image "shared.tga", } }
        Materials 1 { Material { Layer {
            FilterMode None, Shader "Shader_HD_DefaultUnit",
            static TextureID 0 <= 0,
            static TextureID 0 <= 1,
            static TextureID 0 <= 2,
        } } }
    "#,
    )
    .unwrap();
    let mut app = app(&directory);
    let handle = app.world().resource::<AssetServer>().load("unit.mdl");
    wait(&mut app, &handle).unwrap();
    let models = app.world().resource::<Assets<UniversalModelAsset>>();
    let model = models.get(&handle).unwrap();
    assert_eq!(model.images.len(), 2);
    let images = app.world().resource::<Assets<Image>>();
    for (key, handle) in &model.images {
        let image = images.get(handle).unwrap();
        assert_eq!(
            image.texture_descriptor.format.is_srgb(),
            key.color_space == TextureColorSpace::Srgb
        );
        assert!(image.data.as_ref().is_some_and(|bytes| !bytes.is_empty()));
    }
    fs::remove_dir_all(directory).unwrap();
}

fn write_animated_m2(directory: &Path) {
    use wc3::model::conversion::{ExportOptions, ImportOptions};
    use wc3::model::formats::m2::companions::SkinProfile;
    use wc3::model::formats::m2::records::{
        Batch, BlendMode, Bone, Interpolation, LegacyTimeline, LegacyTiming, LegacyValues,
        Material, MaterialFlags, Sequence, SequenceFlags, SkinSection, Texture, TextureFlags,
        TextureSource, Track, TrackBase, Vertex,
    };
    use wc3::model::ZeroVec;
    let mut native = m2::Document::<m2::Flat<m2::V256>>::new();
    let core = native.core_mut();
    core.name = ZeroVec::alloc_from_slice(b"Universal animated triangle\0");
    core.bones = vec![Bone::new()];
    core.vertices = ZeroVec::alloc_from_slice(&[
        Vertex {
            position: [-1., 0., 0.],
            normal: [0., -1., 0.],
            texture_coordinates: [[0., 1.], [0., 0.]],
            bone_indices: [0; 4],
            bone_weights: [255, 0, 0, 0],
        },
        Vertex {
            position: [1., 0., 0.],
            normal: [0., -1., 0.],
            texture_coordinates: [[1., 1.], [1., 0.]],
            bone_indices: [0; 4],
            bone_weights: [255, 0, 0, 0],
        },
        Vertex {
            position: [0., 0., 2.],
            normal: [0., -1., 0.],
            texture_coordinates: [[0.5, 0.], [0.5, 1.]],
            bone_indices: [0; 4],
            bone_weights: [255, 0, 0, 0],
        },
    ]);
    core.textures = vec![Texture {
        filename: ZeroVec::alloc_from_slice(b"checker.tga\0"),
        kind: TextureSource::File,
        flags: TextureFlags::from_bits_retain(3),
    }];
    core.materials = ZeroVec::alloc_from_slice(&[Material {
        flags: MaterialFlags::from_bits_retain(4),
        blending_mode: BlendMode::Opaque,
    }]);
    core.texture_combos = ZeroVec::alloc_from_slice(&[0]);
    core.texture_weight_combos = ZeroVec::alloc_from_slice(&[u16::MAX]);
    core.texture_transform_combos = ZeroVec::alloc_from_slice(&[u16::MAX]);
    core.texture_transform_bone_map = ZeroVec::alloc_from_slice(&[0]);
    core.skins = vec![SkinProfile {
        vertices: ZeroVec::alloc_from_slice(&[0, 1, 2]),
        indices: ZeroVec::alloc_from_slice(&[0, 1, 2]),
        bones: ZeroVec::alloc_from_slice(&[[0; 4]; 3]),
        sections: vec![SkinSection {
            vertex_count: 3,
            index_count: 3,
            id: 17,
            ..Default::default()
        }],
        batches: vec![Batch {
            color_index: -1,
            texture_count: 1,
            shader_id: 0,
            ..Default::default()
        }],
        ..Default::default()
    }];
    core.sequences.push(Sequence {
        timing: LegacyTiming {
            start: 0,
            end: 2000,
        },
        id: 0,
        flags: SequenceFlags::from_bits_retain(0x20),
        variation_next: -1,
        ..Default::default()
    });
    core.bones[0].translation = Track {
        base: TrackBase {
            interpolation: Interpolation::Linear,
            global_sequence: u16::MAX,
            timeline: LegacyTimeline {
                ranges: ZeroVec::alloc_from_slice(&[[0, 2]]),
                timestamps: ZeroVec::alloc_from_slice(&[0, 1000, 2000]),
            },
        },
        values: LegacyValues::Values(ZeroVec::alloc_from_slice(&[
            [0., 0., 0.],
            [0., 0., 0.8],
            [0., 0., 0.],
        ])),
    };
    let bundle = m2::Bundle::from_model(native);
    let imported = semantic_m2::import(&bundle, &ImportOptions::default())
        .unwrap()
        .value;
    let modern = semantic_m2::plan::<m2::Chunked<m2::V274>>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let encoded = modern.encode().unwrap();
    fs::create_dir_all(directory).unwrap();
    fs::write(directory.join("unit.m2"), encoded.model).unwrap();
    for (key, bytes) in encoded.files {
        let name = match key {
            FileKey::Sibling { suffix, .. } => format!("unit{suffix}"),
            FileKey::Path(path) => path,
            other => panic!("unexpected generated resource {other:?}"),
        };
        fs::write(directory.join(name), bytes).unwrap();
    }
    // Four colored quadrants make UV orientation visible in captures.
    let mut tga = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 24, 32];
    tga.extend_from_slice(&[0, 0, 255, 0, 255, 0, 255, 0, 0, 0, 255, 255]);
    fs::write(directory.join("checker.tga"), tga).unwrap();
}
#[test]
fn native_m2_external_skin_compiles_geometry_and_animation() {
    let directory = directory("animated-m2");
    write_animated_m2(&directory);
    let mut app = app(&directory);
    let handle = app.world().resource::<AssetServer>().load("unit.m2");
    wait(&mut app, &handle).unwrap();
    let models = app.world().resource::<Assets<UniversalModelAsset>>();
    let model = models.get(&handle).unwrap();
    assert_eq!(model.compiled.draws.len(), 1);
    assert_eq!(model.compiled.clips.len(), 1);
    assert_eq!(model.images.len(), 1);
    assert!(model.unresolved_images.is_empty());
    fs::remove_dir_all(directory).unwrap();
}
#[test]
#[ignore = "writes the explicit temporary GPU capture model"]
fn write_m2_capture_fixture() {
    write_animated_m2(Path::new("/tmp/wc3-universal-m2"));
}
