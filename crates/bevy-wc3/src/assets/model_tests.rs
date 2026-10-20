use super::*;
use crate::BlpImageLoader;
use bevy::asset::{AssetApp, AssetPlugin, LoadState};
use bevy::image::{ImageAddressMode, ImageLoader, ImageSampler};
use bevy::tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool, TaskPoolBuilder};
use std::collections::HashSet;
use std::fs;
use std::thread::sleep;
use std::time::{Duration, Instant};
use wc3::model::{AddressMode, ModelMetadata};
use wc3::model::common::ZeroVec;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
fn app(directory: &Path) -> App {
    app_with_resolver(directory, Arc::new(PathResolver))
}
fn app_with_resolver(directory: &Path, resolver: Arc<dyn ResourceResolver>) -> App {
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
    app.init_asset::<ModelAsset>();
    app.register_asset_loader(ModelLoader::new(resolver));
    app
}
fn directory(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("universal-{name}-{}", std::process::id()))
}
fn wait(app: &mut App, handle: &Handle<ModelAsset>) -> Result<(), String> {
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
    let resolver = PathResolver;
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
    assert!(
        resolver
            .candidates(&ModelResource::FileDataId(42), &owner)
            .is_empty()
    );
    assert!(
        resolver
            .candidates(&ModelResource::Path(Cow::Borrowed(b"../outside")), &owner)
            .is_empty()
    );
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
    let paths = PathResolver.candidates(&relative, &owner);
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
        Materials 1 { Material {
            Layer { FilterMode None, static TextureID 0, }
            Layer { FilterMode None, static TextureID 1, }
        } }
    "#,
    )
    .unwrap();
    let mut app = app(&directory);
    let handle = app.world().resource::<AssetServer>().load("unit.mdl");
    wait(&mut app, &handle).unwrap();
    let models = app.world().resource::<Assets<ModelAsset>>();
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
    use crate::textures::TextureColorSpace;
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
    let models = app.world().resource::<Assets<ModelAsset>>();
    let model = models.get(&handle).unwrap();
    assert_eq!(
        model
            .images
            .values()
            .map(Handle::id)
            .collect::<HashSet<_>>()
            .len(),
        2
    );
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
    let models = app.world().resource::<Assets<ModelAsset>>();
    let model = models.get(&handle).unwrap();
    assert_eq!(model.compiled.draws.len(), 1);
    assert_eq!(model.compiled.clips.len(), 1);
    assert_eq!(
        model
            .images
            .values()
            .map(Handle::id)
            .collect::<HashSet<_>>()
            .len(),
        1
    );
    assert!(model.unresolved_images.is_empty());
    fs::remove_dir_all(directory).unwrap();
}
#[test]
#[ignore = "writes the explicit temporary GPU capture model"]
fn write_m2_capture_fixture() {
    write_animated_m2(Path::new("/tmp/wc3-universal-m2"));
}

#[test]
fn image_candidates_preserve_location_priority_case_and_asset_sources() {
    let mut slot_values = SlotValues::default();
    use wc3::model::ModelBuilder;
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let id = builder.add_file_texture("Textures\\Body.BLP").unwrap();
    let mut layer = wc3::model::materials::mdlx::MaterialStep::construct(&mut slot_values);
    layer.shader.inputs_mut().binding.image = slot_values.add(Some(id));
    builder
        .add_material(wc3::model::Material {
            name: "image input".into(),
            surface: wc3::model::profiles::mdlx::Material {
                declaration: Default::default(),
                steps: vec![layer],
            }
            .into(),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let request = texture_requests(&model)
        .into_iter()
        .find(|request| request.texture == id)
        .unwrap();
    let paths =
        PathResolver.texture_candidates(0, &request, &AssetPath::parse("casc://Units/Unit.m2"));
    assert_eq!(
        paths.iter().map(ToString::to_string).collect::<Vec<_>>(),
        [
            "casc://Units/Textures/Body.BLP",
            "casc://Units/Textures/Body.blp",
            "casc://Units/Textures/Body.dds",
            "casc://Units/Textures/Body.png",
            "casc://Units/Textures/Body.tga",
            "casc://Textures/Body.BLP",
            "casc://Textures/Body.blp",
            "casc://Textures/Body.dds",
            "casc://Textures/Body.png",
            "casc://Textures/Body.tga",
        ]
    );
    // Companions retain literal lookup; texture fallback does not affect them.
    assert_eq!(
        PathResolver
            .candidates(
                &request.resource.clone().try_into().unwrap(),
                &AssetPath::parse("casc://Units/Unit.m2")
            )
            .len(),
        2
    );
}

struct TextureOverrides;
impl ResourceResolver for TextureOverrides {
    fn candidates(
        &self,
        resource: &ModelResource<'_>,
        owner: &AssetPath<'_>,
    ) -> Vec<AssetPath<'static>> {
        PathResolver.candidates(resource, owner)
    }
    fn texture_resource(
        &self,
        index: usize,
        request: &TextureRequest<'_, '_>,
        _owner: &AssetPath<'_>,
    ) -> ImageResource<'static> {
        assert!(matches!(
            request.resource,
            ImageResource::Replaceable { id: 1, .. }
        ));
        let variant = usize::from(request.sampler.u == wc3::model::AddressMode::Repeat);
        assert_eq!(
            index, variant,
            "native bitmap declarations retain their table identities"
        );
        ImageResource::Path(Cow::Owned(format!("mapped{variant}.blp").into_bytes()))
    }
}

#[test]
fn per_texture_overrides_keep_role_and_sampler_variants_independent() {
    use crate::textures::TextureColorSpace;
    let directory = directory("override-variants");
    fs::create_dir_all(directory.join("units")).unwrap();
    let tga = |pixel: [u8; 3]| {
        let mut bytes = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 24, 32];
        bytes.extend(pixel);
        bytes
    };
    fs::write(directory.join("units/mapped0.tga"), tga([0, 0, 255])).unwrap();
    // Local alternate extension wins over alternate paths at the asset root.
    fs::write(directory.join("mapped0.tga"), tga([0, 255, 0])).unwrap();
    fs::write(directory.join("mapped1.tga"), tga([255, 0, 0])).unwrap();
    fs::write(
        directory.join("units/unit.mdl"),
        r#"
        Version { FormatVersion 1100, } Model "overrides" {}
        Textures 2 {
            Bitmap { Image "", ReplaceableId 1, }
            Bitmap { Image "", ReplaceableId 1, WrapWidth, }
        }
        Materials 1 { Material { Layer {
            FilterMode None, Shader "Shader_HD_DefaultUnit",
            static TextureID 0 <= 0,
            static TextureID 0 <= 1,
            static TextureID 0 <= 2,
        }
        Layer {
            FilterMode None, Shader "Shader_HD_DefaultUnit",
            static TextureID 1 <= 0,
            static TextureID 1 <= 1,
            static TextureID 1 <= 2,
        } } }
    "#,
    )
    .unwrap();
    let mut app = app_with_resolver(&directory, Arc::new(TextureOverrides));
    let handle = app.world().resource::<AssetServer>().load("units/unit.mdl");
    wait(&mut app, &handle).unwrap();
    let models = app.world().resource::<Assets<ModelAsset>>();
    let model = models.get(&handle).unwrap();
    assert!(model.unresolved_images.is_empty());
    // Both active inputs retain their own override and role interpretations.
    let images = app.world().resource::<Assets<Image>>();
    let mut red = 0;
    let mut blue = 0;
    for (key, handle) in &model.images {
        let image = images.get(handle).unwrap();
        assert_eq!(
            image.texture_descriptor.format.is_srgb(),
            key.color_space == TextureColorSpace::Srgb
        );
        let pixel = &image.data.as_ref().unwrap()[..3];
        let ImageSampler::Descriptor(sampler) = &image.sampler else {
            panic!("sampler missing")
        };
        if pixel == [255, 0, 0] {
            red += 1;
            assert_eq!(sampler.address_mode_u, ImageAddressMode::ClampToEdge);
        } else if pixel == [0, 0, 255] {
            blue += 1;
            assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
        } else {
            panic!("wrong lookup location or collapsed override: {pixel:?}");
        }
    }
    assert!(red > 0 && blue > 0);
    assert!(model.images.values().any(|handle| {
        !images
            .get(handle)
            .unwrap()
            .texture_descriptor
            .format
            .is_srgb()
    }));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn child_dependency_graph_rejects_cycles_between_independent_roots() {
    let mut graph = ModelDependencyGraph::default();
    let a = AssetPath::parse("casc://a.m2");
    let b = AssetPath::parse("casc://b.m2");
    let c = AssetPath::parse("casc://c.m2");
    assert!(graph.insert(&a, &b));
    assert!(graph.insert(&b, &c));
    assert!(!graph.insert(&c, &a));
    assert!(!graph.insert(&a, &a));
    assert!(graph.insert(&a, &c));
}

#[test]
fn optional_attachment_models_load_with_extension_fallback_and_stop_cycles() {
    use crate::{ModelInstance, ModelMaterial, spawn_model};
    use bevy::ecs::world::CommandQueue;
    use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
    let directory = directory("child-models");
    fs::create_dir_all(&directory).unwrap();
    let model = |name: &str, child: &str| {
        format!(
            r#"
        Version {{ FormatVersion 800, }} Model "{name}" {{}}
        Attachment "child" {{ ObjectId 0, AttachmentID 0, Path "{child}", }}
        PivotPoints 1 {{ {{0,0,0}}, }}
    "#
        )
    };
    fs::write(directory.join("root.mdl"), model("root", "child.mdl")).unwrap();
    fs::write(directory.join("child.mdx"), model("child", "root.mdl")).unwrap();
    let mut app = app(&directory);
    let root: Handle<ModelAsset> = app.world().resource::<AssetServer>().load("root.mdl");
    wait(&mut app, &root).unwrap();
    let child: Handle<ModelAsset> = app.world().resource::<AssetServer>().load("child.mdx");
    wait(&mut app, &child).unwrap();
    let assets = app.world().resource::<Assets<ModelAsset>>();
    assert_eq!(assets.len(), 2);
    assert!(
        assets
            .get(&child)
            .unwrap()
            .compiled
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.message.contains("cyclic child model reference"))
    );
    let compiled = assets.get(&root).unwrap().compiled.clone();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<ModelMaterial>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        spawn_model(
            &mut commands,
            &mut meshes,
            &mut materials,
            &prepared,
            Transform::default(),
        );
    }
    queue.apply(app.world_mut());
    let mounted: Vec<_> = app
        .world_mut()
        .query::<&ModelInstance>()
        .iter(app.world())
        .map(|instance| instance.0.clone())
        .collect();
    assert_eq!(mounted, vec![child]);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn dependency_reload_replaces_old_outgoing_edges_and_retains_other_roots() {
    let mut graph = ModelDependencyGraph::default();
    let a = AssetPath::parse("a.m2");
    let b = AssetPath::parse("b.m2");
    let c = AssetPath::parse("c.m2");
    let d = AssetPath::parse("d.m2");
    assert!(graph.insert(&a, &b));
    assert!(graph.insert(&d, &c));
    let generation = graph.begin_load(&a);
    assert!(graph.insert(&a, &c));
    graph.finish_load(&a, generation, true);
    assert!(graph.insert(&b, &a));
    assert!(!graph.insert(&c, &b));
    assert!(!graph.insert(&c, &d));
}

#[test]
fn failed_dependency_attempt_discards_partial_edges_before_retry() {
    let graph = Mutex::new(ModelDependencyGraph::default());
    let a = AssetPath::parse("a.mdl");
    let b = AssetPath::parse("b.mdl");
    {
        let _attempt = ModelDependencyLoad::begin(&graph, &a);
        assert!(graph.lock().unwrap().insert(&a, &b));
        // Simulate a native decode/material compilation error after discovery.
    }
    assert!(graph.lock().unwrap().insert(&b, &a));
    let retry = ModelDependencyLoad::begin(&graph, &a);
    assert!(!graph.lock().unwrap().insert(&a, &b));
    retry.commit();
    assert!(!graph.lock().unwrap().edges.contains_key(&a));
}

#[test]
fn successful_dependency_attempt_commits_only_its_current_edges() {
    let graph = Mutex::new(ModelDependencyGraph::default());
    let a = AssetPath::parse("a.mdl");
    let b = AssetPath::parse("b.mdl");
    let attempt = ModelDependencyLoad::begin(&graph, &a);
    assert!(graph.lock().unwrap().insert(&a, &b));
    attempt.commit();
    assert!(!graph.lock().unwrap().insert(&b, &a));
}

#[test]
fn failed_reload_preserves_committed_edges_and_discards_provisional_edges() {
    let graph = Mutex::new(ModelDependencyGraph::default());
    let a = AssetPath::parse("a.mdl");
    let b = AssetPath::parse("b.mdl");
    let c = AssetPath::parse("c.mdl");
    assert!(graph.lock().unwrap().insert(&a, &b));
    {
        let _attempt = ModelDependencyLoad::begin(&graph, &a);
        assert!(graph.lock().unwrap().insert(&a, &c));
        assert!(!graph.lock().unwrap().insert(&b, &a));
        assert!(!graph.lock().unwrap().insert(&c, &a));
    }
    assert!(!graph.lock().unwrap().insert(&b, &a));
    assert!(graph.lock().unwrap().insert(&c, &a));
    assert!(graph.lock().unwrap().pending.is_empty());
}

#[test]
fn stale_reload_attempt_cannot_mutate_or_finish_newer_pending_edges() {
    let graph = Mutex::new(ModelDependencyGraph::default());
    let a = AssetPath::parse("a.mdl");
    let b = AssetPath::parse("b.mdl");
    let c = AssetPath::parse("c.mdl");
    let old = ModelDependencyLoad::begin(&graph, &a);
    let new = ModelDependencyLoad::begin(&graph, &a);
    assert!(
        !graph
            .lock()
            .unwrap()
            .insert_for_load(&a, old.generation, &b)
    );
    assert!(
        graph
            .lock()
            .unwrap()
            .insert_for_load(&a, new.generation, &c)
    );
    drop(old);
    assert!(!graph.lock().unwrap().insert(&c, &a));
    new.commit();
    assert!(!graph.lock().unwrap().insert(&c, &a));
    assert!(graph.lock().unwrap().insert(&b, &a));
}

#[test]
fn native_import_loss_requires_opt_in_and_retains_diagnostic() {
    use wc3::model::conversion::UnknownPolicy;

    struct AllowUnknown;
    impl ResourceResolver for AllowUnknown {
        fn candidates(
            &self,
            resource: &ModelResource<'_>,
            owner: &AssetPath<'_>,
        ) -> Vec<AssetPath<'static>> {
            PathResolver.candidates(resource, owner)
        }
        fn import_options(&self, _owner: &AssetPath<'_>) -> ImportOptions {
            ImportOptions {
                unknown: UnknownPolicy::Drop,
                ..Default::default()
            }
        }
    }

    let directory = directory("native-import-policy");
    write_animated_m2(&directory);
    let mut bytes = fs::read(directory.join("unit.m2")).unwrap();
    bytes.extend_from_slice(b"ZZZZ");
    bytes.extend_from_slice(&4u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 2, 3, 4]);
    fs::write(directory.join("unit.m2"), bytes).unwrap();

    let mut strict = app(&directory);
    let handle = strict.world().resource::<AssetServer>().load("unit.m2");
    let error = wait(&mut strict, &handle).unwrap_err();
    assert!(error.contains("unknown extension"), "{error}");

    let mut permitted = app_with_resolver(&directory, Arc::new(AllowUnknown));
    let handle = permitted.world().resource::<AssetServer>().load("unit.m2");
    wait(&mut permitted, &handle).unwrap();
    let assets = permitted.world().resource::<Assets<ModelAsset>>();
    let asset = assets.get(&handle).unwrap();
    assert!(asset.compiled.diagnostics().iter().any(|diagnostic| {
        diagnostic.message.contains("Dropped") && diagnostic.message.contains("unknown extension")
    }));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn native_world_surface_sampler_variants_get_distinct_labeled_gpu_images() {
    use wc3::model::formats::wmo::{
        Classic, Root, codec::Array, records::Material as NativeMaterial, root::RootChunk,
    };
    struct WorldResolver;
    impl ResourceResolver for WorldResolver {
        fn candidates(
            &self,
            resource: &ModelResource<'_>,
            owner: &AssetPath<'_>,
        ) -> Vec<AssetPath<'static>> {
            PathResolver.candidates(resource, owner)
        }
        fn wmo_profile(&self, _owner: &AssetPath<'_>) -> Option<wmo::Profile> {
            Some(wmo::Profile::Classic)
        }
    }
    let directory = directory("world-surface-samplers");
    fs::create_dir_all(&directory).unwrap();
    let tga = [
        0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 24, 32, 0, 0, 255,
    ];
    fs::write(directory.join("body.tga"), tga).unwrap();
    let repeat = NativeMaterial::<Classic>::default();
    let mut clamp = repeat.clone();
    clamp.flags.set_clamp_s(true);
    let mut root = Root::<Classic>::new();
    root.header_mut().texture_count = 3;
    root.insert_chunk(
        0,
        RootChunk::TextureNames(Array::from_records(b"body.tga\0").unwrap()),
    )
    .unwrap();
    root.insert_chunk(
        1,
        RootChunk::Materials(Array::from_records(&[repeat, clamp.clone(), clamp]).unwrap()),
    )
    .unwrap();
    fs::write(directory.join("unit.wmo"), root.encode().unwrap()).unwrap();
    let mut app = app_with_resolver(&directory, Arc::new(WorldResolver));
    let handle = app.world().resource::<AssetServer>().load("unit.wmo");
    wait(&mut app, &handle).unwrap();
    let model = app
        .world()
        .resource::<Assets<ModelAsset>>()
        .get(&handle)
        .unwrap();
    assert_eq!(
        model
            .images
            .values()
            .map(Handle::id)
            .collect::<HashSet<_>>()
            .len(),
        2
    );
    let images = app.world().resource::<Assets<Image>>();
    let mut modes = Vec::new();
    for (key, handle) in &model.images {
        let ImageSampler::Descriptor(sampler) = &images.get(handle).unwrap().sampler else {
            panic!("missing sampler")
        };
        modes.push(sampler.address_mode_u);
        assert_eq!(
            sampler.address_mode_u,
            if key.sampler.u == AddressMode::Clamp {
                ImageAddressMode::ClampToEdge
            } else {
                ImageAddressMode::Repeat
            }
        );
    }
    assert!(modes.contains(&ImageAddressMode::ClampToEdge));
    assert!(modes.contains(&ImageAddressMode::Repeat));
    fs::remove_dir_all(directory).unwrap();
}
