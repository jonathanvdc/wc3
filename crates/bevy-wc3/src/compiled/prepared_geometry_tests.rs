//! Independent asset lifetime, descriptor and model-behavior regressions.
use super::*;
use crate::image_replacements::{refresh_image_replacements, ReplacementImages};
use crate::instances::{invalidate_models, spawn_loaded_models, PreparedModelCache};
use crate::{
    compile_model, BakeSpace, CompiledModel, ModelAsset, ModelInstance, ModelMaterial, ModelRig,
    Playback, PoseBaker, PoseOptions,
};
use bevy::asset::{Asset, AssetApp, AssetEvent, AssetPlugin, RenderAssetUsages};
use bevy::math::bounding::Aabb3d;
use bevy::mesh::morph::MorphAttributes;
use bevy::mesh::skinning::{JointAabb, JointIndex, SkinnedMeshBounds};
use bevy::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::VertexFormat;
use std::collections::HashMap;
use wc3::model::materials::mdlx::{MaterialInputs, MaterialStep, Shader};
use wc3::model::profiles::mdlx::Material as MdlxMaterial;
use wc3::model::{
    Clip, Curve, Draw, DynamicTrack, ExplicitBinding, GeometryData, GeometryInfluences, Influence,
    Material, MaterialSurface, Mesh as ModelMesh, MeshInstance, Model, ModelMetadata, NodeId, Skin,
    Time, TimeSource, Track, UniversalBinding,
};

fn triangle() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.], [1., 0.], [0., 1.]]);
    mesh.insert_indices(Indices::U16(vec![0, 1, 2]));
    mesh
}
fn collect_dropped<A: Asset>(assets: Assets<A>) -> Assets<A> {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<A>()
        .insert_resource(assets)
        .add_systems(Update, Assets::<A>::track_assets);
    app.update();
    app.world_mut().remove_resource::<Assets<A>>().unwrap()
}

#[test]
fn equal_meshes_share_even_when_owned_by_independent_compilations() {
    let mut cache = PreparedGeometryCache::default();
    let mut meshes = Assets::default();
    let a = cache.mesh(&mut meshes, &triangle());
    let b = cache.mesh(&mut meshes, &triangle());
    assert_eq!(a.id(), b.id());
    assert_eq!(meshes.len(), 1);
    assert!(a.is_strong() && b.is_strong());
    assert_eq!(meshes.get(&a).unwrap(), &triangle());
}

#[test]
fn collision_buckets_compare_all_current_mesh_data_and_descriptors() {
    let mut variants = Vec::new();
    let mut changed = triangle();
    changed.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0., 0., 0.], [2., 0., 0.], [0., 1., 0.]],
    );
    variants.push(("position", changed));
    let mut changed = triangle();
    changed.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
    );
    variants.push(("signed zero", changed));
    let mut changed = triangle();
    changed.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.], [0.5, 0.], [0., 1.]]);
    variants.push(("coordinate", changed));
    let mut changed = triangle();
    changed.insert_attribute(Mesh::ATTRIBUTE_TANGENT, vec![[1., 0., 0., 1.]; 3]);
    variants.push(("additional attribute", changed));
    let mut changed = triangle();
    changed.insert_indices(Indices::U16(vec![0, 2, 1]));
    variants.push(("index bytes", changed));
    let mut changed = triangle();
    changed.insert_indices(Indices::U32(vec![0, 1, 2]));
    variants.push(("index type", changed));
    let mut changed = Mesh::new(
        PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::default(),
    );
    changed.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
    );
    changed.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.], [1., 0.], [0., 1.]]);
    changed.insert_indices(Indices::U16(vec![0, 1, 2]));
    variants.push(("topology", changed));
    let mut changed = triangle();
    changed.asset_usage = RenderAssetUsages::MAIN_WORLD;
    variants.push(("asset usage", changed));
    let mut changed = triangle();
    changed.enable_raytracing = false;
    variants.push(("raytracing descriptor", changed));
    let mut changed = triangle();
    changed.set_morph_targets(vec![
        MorphAttributes::new(Vec3::X, Vec3::ZERO, Vec3::ZERO);
        3
    ]);
    variants.push(("morph displacements", changed));
    let mut changed = triangle();
    changed.set_morph_target_names(vec!["jaw".into()]);
    variants.push(("morph names", changed));
    let mut changed = triangle();
    changed.set_skinned_mesh_bounds(Some(SkinnedMeshBounds::default()));
    variants.push(("skin bounds", changed));
    for (name, changed) in variants {
        let mut cache = PreparedGeometryCache::default();
        let mut meshes = Assets::default();
        let first = cache.mesh_bucket(19, &mut meshes, &triangle());
        let second = cache.mesh_bucket(19, &mut meshes, &changed);
        assert_ne!(
            first.id(),
            second.id(),
            "{name} must not alias despite hash collision"
        );
        assert_eq!(meshes.get(&second).unwrap(), &changed);
        assert_eq!(
            cache.mesh_bucket(19, &mut meshes, &changed).id(),
            second.id()
        );
    }
    // Identical bytes with different declared formats still require different buffers.
    let mut float = triangle();
    float.insert_attribute(
        MeshVertexAttribute::new("custom", 987_654, VertexFormat::Float32),
        vec![0_f32; 3],
    );
    let mut integer = triangle();
    integer.insert_attribute(
        MeshVertexAttribute::new("custom", 987_654, VertexFormat::Uint32),
        vec![0_u32; 3],
    );
    let mut cache = PreparedGeometryCache::default();
    let mut meshes = Assets::default();
    let first = cache.mesh_bucket(23, &mut meshes, &float);
    assert_ne!(
        cache.mesh_bucket(23, &mut meshes, &integer).id(),
        first.id()
    );
}

#[test]
fn numerically_equal_metadata_with_distinct_float_bits_does_not_alias() {
    let mut cases = Vec::new();
    let mut positive = triangle();
    positive.set_morph_targets(vec![
        MorphAttributes::new(Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);
        3
    ]);
    let mut negative = positive.clone();
    negative.set_morph_targets(vec![
        MorphAttributes::new(
            Vec3::new(-0., 0., 0.),
            Vec3::ZERO,
            Vec3::ZERO
        );
        3
    ]);
    cases.push(("morph displacement", positive, negative));
    let mut positive = triangle();
    positive.set_skinned_mesh_bounds(Some(SkinnedMeshBounds {
        aabbs: vec![JointAabb {
            center: Vec3::ZERO,
            half_size: Vec3::new(1., 1., 0.),
        }],
        aabb_index_to_joint_index: vec![JointIndex(0)],
    }));
    let mut negative = positive.clone();
    let mut bounds = positive.skinned_mesh_bounds().unwrap().clone();
    bounds.aabbs[0].center.x = -0.;
    negative.set_skinned_mesh_bounds(Some(bounds));
    cases.push(("skin-bound center", positive.clone(), negative));
    let mut negative = positive.clone();
    let mut bounds = positive.skinned_mesh_bounds().unwrap().clone();
    bounds.aabbs[0].half_size.z = -0.;
    negative.set_skinned_mesh_bounds(Some(bounds));
    cases.push(("skin-bound half size", positive, negative));
    let mut positive = triangle();
    positive.final_aabb = Some(Aabb3d::from_min_max(Vec3::ZERO, Vec3::new(1., 1., 0.)));
    let mut negative = positive.clone();
    negative.final_aabb.as_mut().unwrap().min.x = -0.;
    cases.push(("final bounds minimum", positive.clone(), negative));
    let mut negative = positive.clone();
    negative.final_aabb.as_mut().unwrap().max.z = -0.;
    cases.push(("final bounds maximum", positive, negative));
    for (name, positive, negative) in cases {
        assert_eq!(
            positive, negative,
            "the fixture isolates a bit difference hidden by Bevy equality"
        );
        let mut cache = PreparedGeometryCache::default();
        let mut meshes = Assets::default();
        let first = cache.mesh_bucket(31, &mut meshes, &positive);
        let second = cache.mesh_bucket(31, &mut meshes, &negative);
        assert_ne!(
            first.id(),
            second.id(),
            "{name} requires distinct stored data"
        );
        assert_eq!(
            cache.mesh_bucket(31, &mut meshes, &positive).id(),
            first.id()
        );
        assert_eq!(
            cache.mesh_bucket(31, &mut meshes, &negative).id(),
            second.id()
        );
    }
}

#[test]
fn palette_sharing_preserves_order_length_and_exact_matrix_bits() {
    let original = [Mat4::IDENTITY, Mat4::from_translation(Vec3::X)];
    let mut cache = PreparedGeometryCache::default();
    let mut assets = Assets::default();
    let first = cache.palette_bucket(5, &mut assets, &original);
    assert_eq!(
        cache.palette_bucket(5, &mut assets, &original).id(),
        first.id()
    );
    for changed in [
        vec![original[1], original[0]],
        vec![original[0]],
        vec![original[0], Mat4::from_translation(Vec3::Y)],
        {
            let mut signed = original;
            signed[0].x_axis.y = -0.;
            signed.to_vec()
        },
    ] {
        let second = cache.palette_bucket(5, &mut assets, &changed);
        assert_ne!(second.id(), first.id());
        assert_eq!(assets.get(&second).unwrap().as_ref(), changed.as_slice());
    }
    let empty = cache.palette(&mut assets, &[]);
    assert!(assets.get(&empty).unwrap().is_empty());
    assert_eq!(cache.palette(&mut assets, &[]).id(), empty.id());
}

#[test]
fn mutated_or_removed_assets_are_not_reused_for_later_models() {
    let mut cache = PreparedGeometryCache::default();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let old = cache.mesh(&mut meshes, &triangle());
    meshes
        .get_mut(&old)
        .unwrap()
        .insert_indices(Indices::U16(vec![0, 2, 1]));
    let repaired = cache.mesh(&mut meshes, &triangle());
    assert_ne!(old.id(), repaired.id());
    assert_eq!(meshes.get(&repaired).unwrap(), &triangle());
    assert_eq!(
        meshes.get(&old).unwrap().indices(),
        Some(&Indices::U16(vec![0, 2, 1]))
    );
    let palette = cache.palette(&mut binds, &[Mat4::IDENTITY]);
    binds
        .insert(palette.id(), vec![Mat4::from_translation(Vec3::Z)].into())
        .unwrap();
    let repaired_palette = cache.palette(&mut binds, &[Mat4::IDENTITY]);
    assert_ne!(palette.id(), repaired_palette.id());
    meshes.remove(repaired.id()).unwrap();
    binds.remove(repaired_palette.id()).unwrap();
    let recreated = cache.mesh(&mut meshes, &triangle());
    let recreated_palette = cache.palette(&mut binds, &[Mat4::IDENTITY]);
    assert_ne!(recreated.id(), repaired.id());
    assert_ne!(recreated_palette.id(), repaired_palette.id());
    meshes.remove(old.id());
    meshes.remove(recreated.id());
    binds.remove(palette.id());
    binds.remove(recreated_palette.id());
    cache.prune(&meshes, &binds);
    assert!(cache.meshes.is_empty() && cache.palettes.is_empty());
}

#[test]
fn cache_ids_do_not_keep_meshes_or_palettes_alive_after_last_owner_drops() {
    let mut cache = PreparedGeometryCache::default();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let mesh = cache.mesh(&mut meshes, &triangle());
    let mesh_id = mesh.id();
    let palette = cache.palette(&mut binds, &[Mat4::IDENTITY]);
    let palette_id = palette.id();
    drop((mesh, palette));
    let meshes = collect_dropped(meshes);
    let binds = collect_dropped(binds);
    assert!(
        !meshes.contains(mesh_id),
        "the cache must not own a strong mesh handle"
    );
    assert!(
        !binds.contains(palette_id),
        "the cache must not own a strong palette handle"
    );
    cache.prune(&meshes, &binds);
    assert!(cache.meshes.is_empty() && cache.palettes.is_empty());
}

#[test]
fn colliding_asset_ids_in_replacement_stores_upgrade_only_local_ownership() {
    let mut cache = PreparedGeometryCache::default();
    let mut first = Assets::default();
    let old = cache.mesh(&mut first, &triangle());
    let mut replacement = Assets::default();
    let seed = replacement.add(triangle());
    assert_eq!(
        old.id(),
        seed.id(),
        "fixture needs an actual cross-store ID collision"
    );
    let local = cache.mesh(&mut replacement, &triangle());
    drop(seed);
    let replacement = collect_dropped(replacement);
    assert!(
        replacement.contains(local.id()),
        "returned handle must retain the replacement store's asset"
    );
    let local_id = local.id();
    drop(local);
    let replacement = collect_dropped(replacement);
    assert!(
        !replacement.contains(local_id),
        "an owner from the old store must not retain the replacement asset"
    );
    assert!(first.contains(old.id()));
    drop(old);
    assert!(collect_dropped(first).is_empty());
    // A colliding ID whose current payload differs must allocate a new local asset.
    let mut replacement = Assets::default();
    let mut changed = triangle();
    changed.insert_indices(Indices::U16(vec![0, 2, 1]));
    let seed = replacement.add(changed);
    let local = cache.mesh(&mut replacement, &triangle());
    assert_ne!(local.id(), seed.id());
    assert_eq!(replacement.get(&local).unwrap(), &triangle());

    let mut first = Assets::default();
    let old = cache.palette(&mut first, &[Mat4::IDENTITY]);
    let mut replacement = Assets::<SkinnedMeshInverseBindposes>::default();
    let seed = replacement.add(vec![Mat4::IDENTITY]);
    assert_eq!(old.id(), seed.id());
    let local = cache.palette(&mut replacement, &[Mat4::IDENTITY]);
    drop(seed);
    let replacement = collect_dropped(replacement);
    assert!(replacement.contains(local.id()));
    let local_id = local.id();
    drop(local);
    assert!(!collect_dropped(replacement).contains(local_id));
    assert!(first.contains(old.id()));
    drop(old);
    assert!(collect_dropped(first).is_empty());
}

fn compiled(name: &str, opacity: f32, travel: f32) -> (CompiledModel, NodeId) {
    let mut draft = Model::builder(ModelMetadata::Mdlx(Default::default()));
    let node = draft.add_named_node(name.to_owned()).unwrap();
    let target = draft.model().node(node).unwrap().transform.translation;
    let skin = draft
        .add_skin(Skin::new(
            vec![node],
            UniversalBinding::Explicit(ExplicitBinding::new([Mat4::IDENTITY]).unwrap()),
        ))
        .unwrap();
    let mut geometry =
        GeometryData::triangles([Vec3::ZERO, Vec3::X, Vec3::Y], [[0, 1, 2]]).unwrap();
    geometry.set_texture_coordinates(vec![[Vec2::ZERO, Vec2::X, Vec2::Y].into_iter().collect()]);
    let influences = [
        Influence {
            joint: 0,
            weight: 1.,
        },
        Influence {
            joint: 0,
            weight: 0.,
        },
        Influence {
            joint: 0,
            weight: 0.,
        },
        Influence {
            joint: 0,
            weight: 0.,
        },
    ];
    geometry.influences = GeometryInfluences::Four(vec![influences; 3].into_iter().collect());
    let mut mesh = ModelMesh::new(geometry);
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = draft.add_mesh(mesh).unwrap();
    let mut step: MaterialStep = draft.construct();
    let inputs = MaterialInputs {
        opacity: draft.add_value(opacity),
        ..draft.construct()
    };
    step.shader = Shader::Classic(inputs);
    let material = draft
        .add_material(Material {
            name: name.to_owned().into(),
            surface: MaterialSurface::from(MdlxMaterial {
                declaration: Default::default(),
                steps: vec![step],
            }),
        })
        .unwrap();
    let mut instance = MeshInstance::new(node, mesh);
    instance.skin = Some(skin);
    instance.draws.push(Draw::new(subset, Default::default()));
    draft
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    draft
        .add_global_track(DynamicTrack::Vector3(Track {
            target,
            clock: TimeSource::Animation,
            curve: Curve::linear(
                [Time::default(), Time::seconds(1.).unwrap()],
                [Vec3::ZERO, Vec3::Z * travel],
            )
            .unwrap(),
        }))
        .unwrap();
    draft
        .add_clip(Clip::named(name.to_owned()).with_duration(Time::seconds(1.).unwrap()))
        .unwrap();
    (
        compile_model(&draft.finish().unwrap(), |_| None).unwrap(),
        node,
    )
}

#[test]
fn shared_preparation_reuses_buffers_without_merging_model_identity_or_animation() {
    let (a, a_node) = compiled("first", 0.25, 2.);
    let (b, b_node) = compiled("second", 0.75, 6.);
    let mut cache = PreparedGeometryCache::default();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let a = a.prepare_with_cache(&mut meshes, &mut binds, &mut cache);
    let b = b.prepare_with_cache(&mut meshes, &mut binds, &mut cache);
    assert_eq!(meshes.len(), 1);
    assert_eq!(binds.len(), 1);
    assert_eq!(a.draws[0].mesh.id(), b.draws[0].mesh.id());
    assert_eq!(a.skins[0].binds.id(), b.skins[0].binds.id());
    assert_ne!(a_node, b_node);
    assert_eq!(a.nodes[0].id, a_node);
    assert_eq!(b.nodes[0].id, b_node);
    assert_ne!(a.draws[0].source, b.draws[0].source);
    assert_ne!(a.draws[0].material_id, b.draws[0].material_id);
    assert_eq!(a.clips()[0].1, "first");
    assert_eq!(b.clips()[0].1, "second");
    assert_ne!(a.clips()[0].0, b.clips()[0].0);
    for (prepared, alpha, z) in [(&a, 0.25, 1.), (&b, 0.75, 3.)] {
        let mut playback = Playback::for_clip(prepared.clips()[0].0);
        playback.seek(0.5).unwrap();
        let baked = PoseBaker::default()
            .bake(
                prepared,
                &meshes,
                &binds,
                PoseOptions {
                    playback: Some(&playback),
                    space: BakeSpace::Authored,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            baked.parts[0]
                .material
                .as_ref()
                .unwrap()
                .base
                .base_color
                .alpha(),
            alpha
        );
        let positions = baked.parts[0]
            .mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        assert!(positions.iter().all(|point| point[2] == z));
    }
}

#[test]
fn public_prepare_keeps_its_existing_fresh_store_behavior() {
    let (compiled, _) = compiled("manual", 1., 1.);
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let a = compiled.prepare(&mut meshes, &mut binds);
    let b = compiled.prepare(&mut meshes, &mut binds);
    assert_ne!(a.draws[0].mesh.id(), b.draws[0].mesh.id());
    assert_ne!(a.skins[0].binds.id(), b.skins[0].binds.id());
}

fn app() -> App {
    let mut app = App::new();
    app.init_resource::<Assets<ModelAsset>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<SkinnedMeshInverseBindposes>>()
        .init_resource::<Assets<ModelMaterial>>()
        .init_resource::<PreparedModelCache>()
        .init_resource::<PreparedGeometryCache>()
        .init_resource::<ReplacementImages>()
        .init_resource::<Assets<Image>>()
        .add_message::<AssetEvent<ModelAsset>>()
        .add_message::<AssetEvent<Image>>()
        .add_systems(
            Update,
            (
                invalidate_models,
                refresh_image_replacements,
                spawn_loaded_models,
            )
                .chain(),
        );
    app
}
fn asset(compiled: CompiledModel) -> ModelAsset {
    ModelAsset {
        compiled,
        images: HashMap::new(),
        unresolved_images: Vec::new(),
    }
}

#[test]
fn different_model_assets_share_prepared_buffers_across_reload_and_eviction() {
    let (a, a_node) = compiled("first", 0.25, 2.);
    let (b, b_node) = compiled("second", 0.75, 6.);
    let mut app = app();
    let a = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .add(asset(a));
    let b = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .add(asset(b));
    let first = app.world_mut().spawn(ModelInstance(a.clone())).id();
    let second = app.world_mut().spawn(ModelInstance(b.clone())).id();
    app.update();
    let a_model = app.world().get::<ModelRig>(first).unwrap().model();
    let b_model = app.world().get::<ModelRig>(second).unwrap().model();
    let mesh = a_model.draws[0].mesh.id();
    let palette = a_model.skins[0].binds.id();
    assert_eq!(mesh, b_model.draws[0].mesh.id());
    assert_eq!(palette, b_model.skins[0].binds.id());
    assert!(app
        .world()
        .get::<ModelRig>(first)
        .unwrap()
        .node(a_node)
        .is_some());
    assert!(app
        .world()
        .get::<ModelRig>(second)
        .unwrap()
        .node(b_node)
        .is_some());
    assert!(app
        .world()
        .get::<ModelRig>(first)
        .unwrap()
        .node(b_node)
        .is_none());
    let (reloaded, reloaded_node) = compiled("reloaded", 0.5, 4.);
    app.world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .insert(a.id(), asset(reloaded))
        .unwrap();
    app.world_mut()
        .write_message(AssetEvent::<ModelAsset>::Modified { id: a.id() });
    app.update();
    assert_eq!(
        app.world().get::<ModelRig>(first).unwrap().model().draws[0]
            .mesh
            .id(),
        mesh
    );
    assert_eq!(
        app.world().get::<ModelRig>(second).unwrap().model().skins[0]
            .binds
            .id(),
        palette
    );
    let reloaded = app.world().get::<ModelRig>(first).unwrap();
    assert!(reloaded.node(reloaded_node).is_some());
    assert!(reloaded.node(a_node).is_none());
    assert_eq!(
        reloaded.model().draws[0].material.base.base_color.alpha(),
        0.5
    );
    assert_eq!(
        app.world().get::<ModelRig>(second).unwrap().model().draws[0]
            .material
            .base
            .base_color
            .alpha(),
        0.75
    );
    app.world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .remove(a.id())
        .unwrap();
    app.world_mut()
        .write_message(AssetEvent::<ModelAsset>::Removed { id: a.id() });
    app.world_mut().despawn(first);
    app.update();
    assert_eq!(
        app.world().get::<ModelRig>(second).unwrap().model().draws[0]
            .mesh
            .id(),
        mesh
    );
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
    app.world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .remove(b.id())
        .unwrap();
    app.world_mut()
        .write_message(AssetEvent::<ModelAsset>::Removed { id: b.id() });
    app.world_mut().despawn(second);
    app.update();
    let meshes = app.world_mut().remove_resource::<Assets<Mesh>>().unwrap();
    let binds = app
        .world_mut()
        .remove_resource::<Assets<SkinnedMeshInverseBindposes>>()
        .unwrap();
    assert!(collect_dropped(meshes).is_empty());
    assert!(collect_dropped(binds).is_empty());
}
