use super::super::lod::{update_universal_lod, UniversalLodSettings};
use super::*;
use bevy::mesh::VertexAttributeValues;
use wc3::model::profiles::Universal;
use wc3::model::{controls::world::WorldSystem, ModelBuilder};

fn system_id() -> SystemId {
    ModelBuilder::<Universal>::new("world identities")
        .add_system(System::Wmo(WorldSystem::default()))
        .unwrap()
}

#[test]
fn liquid_grid_uses_absolute_heights_and_omits_hidden_tiles() {
    let mut liquid = Liquid {
        position: [10., 20., 99.],
        dimensions: [2, 2],
        vertices: [2., 3., 4., 5.]
            .map(|height| LiquidVertex::Water {
                height,
                velocities: [0; 2],
                first_percentage: 0,
                control: 0,
            })
            .into_iter()
            .collect(),
        tiles: [0].into_iter().collect(),
        ..Default::default()
    };
    let mesh = liquid_mesh(&liquid, [2., 3.]).unwrap().unwrap();
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("missing positions")
    };
    assert_eq!(
        positions,
        &[
            [10., 20., 2.],
            [12., 20., 3.],
            [10., 23., 4.],
            [12., 23., 5.]
        ]
    );
    assert_eq!(mesh.indices().unwrap().len(), 6);
    liquid.tiles = [8].into_iter().collect();
    assert!(liquid_mesh(&liquid, [2., 3.]).unwrap().is_none());
}

#[test]
fn malformed_liquid_grid_returns_an_error() {
    let liquid = Liquid {
        dimensions: [2, 2],
        ..Default::default()
    };
    assert!(liquid_mesh(&liquid, [1.; 2]).is_err());
}

#[test]
fn explicit_portal_reachability_is_conservative_and_keeps_other_systems() {
    let system = system_id();
    let other = system_id();
    let a = GroupId { system, index: 0 };
    let b = GroupId { system, index: 1 };
    let isolated = GroupId { system, index: 2 };
    let runtime = WorldRuntime {
        neighbors: HashMap::from([(a, vec![b]), (b, vec![a]), (isolated, vec![])]),
        ..Default::default()
    };
    let selected = runtime.visibility(&UniversalWorldSelection {
        portals: WorldPortalPolicy::ReachableFrom(a),
        ..Default::default()
    });
    assert!(selected.group_visible(a));
    assert!(selected.group_visible(b));
    assert!(!selected.group_visible(isolated));
    assert!(selected.group_visible(GroupId {
        system: other,
        index: 2
    }));
    let all = runtime.visibility(&UniversalWorldSelection::default());
    assert!(all.group_visible(isolated));
    let unknown = runtime.visibility(&UniversalWorldSelection {
        portals: WorldPortalPolicy::ReachableFrom(GroupId { system, index: 99 }),
        ..Default::default()
    });
    assert!(unknown.group_visible(isolated));
}

#[test]
fn explicit_doodad_selection_keeps_common_set() {
    let system = system_id();
    let common = DoodadSetId { system, index: 0 };
    let chosen = DoodadSetId { system, index: 2 };
    let runtime = WorldRuntime::default();
    let selected = runtime.visibility(&UniversalWorldSelection {
        doodad_sets: Some(HashSet::from([chosen])),
        ..Default::default()
    });
    assert!(selected.set_visible(common));
    assert!(selected.set_visible(chosen));
    assert!(!selected.set_visible(DoodadSetId { system, index: 1 }));
}

#[test]
fn spherical_volume_fades_without_mutating_scene_lighting() {
    let volume = Volume::Sphere {
        center: [0.; 3],
        radii: [2., 4.],
    };
    assert_eq!(world_volume_weight(&volume, Vec3::ZERO), 1.);
    assert_eq!(world_volume_weight(&volume, Vec3::new(3., 0., 0.)), 0.5);
    assert_eq!(world_volume_weight(&volume, Vec3::new(5., 0., 0.)), 0.);
}

#[test]
fn compiled_world_spawns_liquid_hooks_and_selection_gates() {
    use super::super::{
        compile_universal_model, spawn_universal_model, UniversalMaterial, UniversalPoseBaker,
        UniversalPoseOptions,
    };
    use bevy::ecs::world::CommandQueue;
    use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
    use wc3::model::controls::world::{Group, RegionControls};
    use wc3::model::{GeometryData, Mesh as ModelMesh, MeshInstance, Node};

    let mut builder = ModelBuilder::<Universal>::new("liquid region");
    let node = builder.add_node(Node::named("region")).unwrap();
    let geometry =
        GeometryData::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]]).unwrap();
    let mesh = builder.add_mesh(ModelMesh::new(geometry)).unwrap();
    let mut placement = MeshInstance::new(node, mesh);
    placement.lod = Some(2);
    placement.controls = Some(InstanceControls::Wmo(Box::new(RegionControls {
        liquids: vec![Liquid {
            dimensions: [2, 2],
            vertices: [0.; 4]
                .map(|height| LiquidVertex::Water {
                    height,
                    velocities: [0; 2],
                    first_percentage: 0,
                    control: 0,
                })
                .into_iter()
                .collect(),
            tiles: [0].into_iter().collect(),
            liquid_type: 42,
            ..Default::default()
        }],
        ..Default::default()
    })));
    let instance = builder.add_instance(placement).unwrap();
    let system = builder
        .add_system(System::Wmo(WorldSystem {
            groups: vec![Group {
                instances: vec![instance],
                ..Default::default()
            }],
            ..Default::default()
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let compiled = compile_universal_model(&model, |_| None).unwrap();
    assert_eq!(compiled.world.groups.len(), 1);
    assert_eq!(compiled.liquids.len(), 1);
    assert!(compiled.liquids[0].surface.is_none());
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    assert_eq!(prepared.world_liquids()[0].liquid_type, 42);
    // Baking uses exact requested authored levels, including liquid-only levels.
    for (lod, expected) in [(None, true), (Some(0), false), (Some(2), true)] {
        let baked = UniversalPoseBaker::default()
            .bake(
                &prepared,
                &meshes,
                &binds,
                UniversalPoseOptions {
                    lod,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(baked.parts.len(), 1);
        assert_eq!(baked.parts[0].visible, expected);
        assert_eq!(baked.bounds.is_some(), expected);
    }
    let mut materials = Assets::<UniversalMaterial>::default();
    let mut app = App::new();
    let mut queue = CommandQueue::default();
    let root = {
        let mut commands = Commands::new(&mut queue, app.world());
        spawn_universal_model(
            &mut commands,
            &mut meshes,
            &mut materials,
            &prepared,
            Transform::default(),
        )
    };
    queue.apply(app.world_mut());
    app.init_resource::<UniversalLodSettings>();
    app.add_systems(Startup, update_universal_lod);
    app.add_systems(Update, apply_universal_world_selection);
    app.world_mut()
        .get_mut::<UniversalPlayback>(root)
        .unwrap()
        .lod = 2;
    app.update();
    assert_eq!(app.world().get::<UniversalPlayback>(root).unwrap().lod, 2);
    let liquid = app
        .world_mut()
        .query_filtered::<Entity, With<UniversalWorldLiquidSurface>>()
        .single(app.world())
        .unwrap();
    assert!(app
        .world()
        .get::<MeshMaterial3d<UniversalMaterial>>(liquid)
        .is_none());
    let gate = app.world().get::<ChildOf>(liquid).unwrap().parent();
    assert_eq!(
        *app.world().get::<Visibility>(gate).unwrap(),
        Visibility::Inherited
    );
    // Selection acts on the parent; authored child visibility remains intact.
    app.world_mut()
        .entity_mut(liquid)
        .insert(Visibility::Hidden);
    app.world_mut()
        .get_mut::<UniversalWorldSelection>(root)
        .unwrap()
        .hidden_groups
        .insert(GroupId { system, index: 0 });
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(gate).unwrap(),
        Visibility::Hidden
    );
    app.world_mut()
        .get_mut::<UniversalWorldSelection>(root)
        .unwrap()
        .hidden_groups
        .clear();
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(gate).unwrap(),
        Visibility::Inherited
    );
    assert_eq!(
        *app.world().get::<Visibility>(liquid).unwrap(),
        Visibility::Hidden
    );
    app.world_mut()
        .get_mut::<UniversalPlayback>(root)
        .unwrap()
        .lod = 0;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(gate).unwrap(),
        Visibility::Hidden
    );
}

#[test]
fn default_placement_selection_includes_common_without_combining_variants() {
    let system = system_id();
    let runtime = WorldRuntime::default();
    let common = DoodadSetId { system, index: 0 };
    let alternative = DoodadSetId { system, index: 1 };
    let selected = runtime.visibility(&UniversalWorldSelection::default());
    assert!(selected.set_visible(common));
    assert!(!selected.set_visible(alternative));
    let all = runtime.visibility(&UniversalWorldSelection {
        doodad_sets: None,
        ..Default::default()
    });
    assert!(all.set_visible(alternative));
}
