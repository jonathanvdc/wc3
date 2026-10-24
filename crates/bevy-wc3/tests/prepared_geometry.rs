//! The public preparation cache shares storage without sharing authored behavior.
use bevy::asset::{Asset, AssetApp, AssetPlugin};
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::CommandQueue;
use bevy::image::ImagePlugin;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy::shader::Shader as ShaderAsset;
use bevy::time::TimeUpdateStrategy;
use bevy_wc3::{
    compile_model, spawn_model, BakeSpace, CompiledModel, DrawSource, ModelAsset, ModelInstance,
    ModelMaterial, ModelRig, Playback, PoseBaker, PoseOptions, PreparedGeometryCache,
    PreparedModel, Wc3BevyPlugin,
};
use std::collections::HashMap;
use std::time::Duration;
use wc3::model::materials::mdlx::{MaterialInputs, MaterialStep, Shader};
use wc3::model::profiles::mdlx::Material as MdlxMaterial;
use wc3::model::{
    Clip, ClipId, Curve, Draw, DrawId, DynamicTrack, ExplicitBinding, GeometryData,
    GeometryInfluences, Influence, InstanceId, Material, MaterialId, MaterialSurface,
    Mesh as ModelMesh, MeshId, MeshInstance, Model, ModelMetadata, NodeId, Skin, Time, TimeSource,
    Track, UniversalBinding,
};

struct Fixture {
    compiled: CompiledModel,
    node: NodeId,
    instance: InstanceId,
    mesh: MeshId,
    material: MaterialId,
    draw: DrawId,
    clip: ClipId,
}
fn fixture(name: &str, opacity: f32, travel: f32) -> Fixture {
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
    let model = draft.finish().unwrap();
    let (instance, placement) = model.instances().iter().next().unwrap();
    let clip = model.clips().iter().next().unwrap().0;
    Fixture {
        compiled: compile_model(&model, |_| None).unwrap(),
        node,
        instance,
        mesh,
        material,
        draw: placement.draws[0].id(),
        clip,
    }
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

fn assert_behavior(
    fixture: &Fixture,
    prepared: &PreparedModel,
    meshes: &Assets<Mesh>,
    binds: &Assets<SkinnedMeshInverseBindposes>,
    alpha: f32,
    height: f32,
) {
    let part = prepared.parts().next().unwrap();
    assert_eq!(part.source.instance, fixture.instance);
    assert_eq!(part.source.mesh, fixture.mesh);
    assert_eq!(part.source.material, fixture.material);
    assert_eq!(part.source.source, DrawSource::Draw(fixture.draw));
    assert_eq!(prepared.clips()[0].0, fixture.clip);
    let mut playback = Playback::for_clip(fixture.clip);
    playback.seek(0.5).unwrap();
    let baked = PoseBaker::default()
        .bake(
            prepared,
            meshes,
            binds,
            PoseOptions {
                playback: Some(&playback),
                space: BakeSpace::Authored,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(baked.parts.len(), 1);
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
    assert!(baked.parts[0]
        .mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .all(|point| point[2] == height));
}

#[test]
fn cache_shares_buffers_while_models_keep_their_ids_materials_and_animation() {
    let first = fixture("first", 0.25, 2.);
    let second = fixture("second", 0.75, 6.);
    assert_ne!(first.node, second.node);
    assert_ne!(first.mesh, second.mesh);
    assert_ne!(first.material, second.material);
    assert_ne!(first.draw, second.draw);
    assert_ne!(first.clip, second.clip);
    let mut cache = PreparedGeometryCache::default();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let a = first
        .compiled
        .prepare_with_cache(&mut meshes, &mut binds, &mut cache);
    let b = second
        .compiled
        .prepare_with_cache(&mut meshes, &mut binds, &mut cache);
    assert_eq!(
        a.parts().next().unwrap().mesh.id(),
        b.parts().next().unwrap().mesh.id()
    );
    assert_eq!(meshes.len(), 1);
    assert_eq!(
        binds.len(),
        1,
        "equal inverse-bind palettes share one asset"
    );
    drop(cache);
    assert_behavior(&first, &a, &meshes, &binds, 0.25, 1.);
    assert_behavior(&second, &b, &meshes, &binds, 0.75, 3.);

    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut materials = Assets::<ModelMaterial>::default();
    let roots = {
        let mut commands = Commands::new(&mut queue, &world);
        [&a, &b].map(|model| {
            spawn_model(
                &mut commands,
                &mut meshes,
                &mut materials,
                model,
                Transform::default(),
            )
        })
    };
    queue.apply(&mut world);
    let a = world.get::<ModelRig>(roots[0]).unwrap();
    let b = world.get::<ModelRig>(roots[1]).unwrap();
    assert!(a.node(first.node).is_some() && a.node(second.node).is_none());
    assert!(b.node(second.node).is_some() && b.node(first.node).is_none());
}

#[test]
fn keeping_cache_alive_does_not_keep_unowned_assets_alive() {
    let fixture = fixture("collectible", 1., 2.);
    let mut cache = PreparedGeometryCache::default();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = fixture
        .compiled
        .prepare_with_cache(&mut meshes, &mut binds, &mut cache);
    let mesh_id = prepared.parts().next().unwrap().mesh.id();
    let palette_id = binds.iter().next().unwrap().0;
    drop(prepared);
    let mut meshes = collect_dropped(meshes);
    let mut binds = collect_dropped(binds);
    assert!(!meshes.contains(mesh_id));
    assert!(!binds.contains(palette_id));
    assert!(meshes.is_empty() && binds.is_empty());
    let recreated = fixture
        .compiled
        .prepare_with_cache(&mut meshes, &mut binds, &mut cache);
    assert_ne!(recreated.parts().next().unwrap().mesh.id(), mesh_id);
    assert_eq!(meshes.len(), 1);
    assert_eq!(binds.len(), 1);
    assert_behavior(&fixture, &recreated, &meshes, &binds, 1., 1.);
    drop(recreated);
    assert!(collect_dropped(meshes).is_empty());
    assert!(collect_dropped(binds).is_empty());
    drop(cache);
}

#[test]
fn ordinary_prepare_still_allocates_fresh_assets_beside_an_explicit_cache() {
    let fixture = fixture("manual", 1., 2.);
    let mut cache = PreparedGeometryCache::default();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let cached = fixture
        .compiled
        .prepare_with_cache(&mut meshes, &mut binds, &mut cache);
    let first = fixture.compiled.prepare(&mut meshes, &mut binds);
    let second = fixture.compiled.prepare(&mut meshes, &mut binds);
    let mesh_ids = [&cached, &first, &second].map(|model| model.parts().next().unwrap().mesh.id());
    assert_ne!(mesh_ids[0], mesh_ids[1]);
    assert_ne!(mesh_ids[1], mesh_ids[2]);
    assert_ne!(mesh_ids[0], mesh_ids[2]);
    assert_eq!(meshes.len(), 3);
    assert_eq!(binds.len(), 3);
    for model in [&cached, &first, &second] {
        assert_behavior(&fixture, model, &meshes, &binds, 1., 1.);
    }
}

#[test]
fn manual_preparation_and_plugin_instances_use_the_same_application_cache() {
    let first = fixture("manual source", 0.25, 2.);
    let second = fixture("automatic source", 0.75, 6.);
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        VisibilityPlugin,
        MeshPlugin,
        ImagePlugin::default(),
    ));
    app.init_asset::<ShaderAsset>();
    app.init_resource::<PreparedGeometryCache>();
    let manual = app
        .world_mut()
        .resource_scope(|world, mut cache: Mut<PreparedGeometryCache>| {
            world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                let mut binds = world.resource_mut::<Assets<SkinnedMeshInverseBindposes>>();
                first
                    .compiled
                    .prepare_with_cache(&mut meshes, &mut binds, &mut cache)
            })
        });
    // Adding the renderer preserves the already populated application resource.
    app.add_plugins(Wc3BevyPlugin);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    let handle = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .add(ModelAsset {
            compiled: second.compiled.clone(),
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        });
    let root = app.world_mut().spawn(ModelInstance(handle.clone())).id();
    app.update();
    let rig = app.world().get::<ModelRig>(root).unwrap();
    assert!(rig.node(second.node).is_some() && rig.node(first.node).is_none());
    let automatic = rig.model();
    assert_eq!(
        manual.parts().next().unwrap().mesh.id(),
        automatic.parts().next().unwrap().mesh.id()
    );
    let meshes = app.world().resource::<Assets<Mesh>>();
    let binds = app
        .world()
        .resource::<Assets<SkinnedMeshInverseBindposes>>();
    assert_eq!(meshes.len(), 1);
    assert_eq!(binds.len(), 1);
    assert_behavior(&first, &manual, meshes, binds, 0.25, 1.);
    assert_behavior(&second, automatic, meshes, binds, 0.75, 3.);
    drop(manual);
    drop(handle);
    app.world_mut().despawn(root);
    // Asset drop messages, source invalidation and GPU-handle drops cross schedules.
    for _ in 0..6 {
        app.update();
    }
    assert!(app.world().resource::<Assets<ModelAsset>>().is_empty());
    assert!(app.world().resource::<Assets<Mesh>>().is_empty());
    assert!(app
        .world()
        .resource::<Assets<SkinnedMeshInverseBindposes>>()
        .is_empty());
    assert!(app.world().contains_resource::<PreparedGeometryCache>());
}
