use super::super::lod::{update_lod, LodSettings};
use super::*;
use bevy::math::{Vec2, Vec3, Vec4};
use bevy::mesh::VertexAttributeValues;
use wc3::model::common::PackedArray;
use wc3::model::profiles::Universal;
use wc3::model::ModelMetadata;
use wc3::model::{profiles::wmo::WorldSystem, ModelBuilder};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
fn system_id() -> SystemId {
    ModelBuilder::<Universal>::new(ModelMetadata::Wmo(Default::default()))
        .add_system(System::Wmo(WorldSystem::default()))
        .unwrap()
}

#[test]
fn liquid_grid_uses_absolute_heights_and_omits_hidden_tiles() {
    let mut slot_values = SlotValues::default();
    let mut liquid = Liquid {
        position: Vec3::from_array([10., 20., 99.]),
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
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mesh = liquid_mesh(&liquid, Vec2::from_array([2., 3.]))
        .unwrap()
        .unwrap();
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
    assert!(liquid_mesh(&liquid, Vec2::from_array([2., 3.]))
        .unwrap()
        .is_none());
}

#[test]
fn malformed_liquid_grid_returns_an_error() {
    let mut slot_values = SlotValues::default();
    let liquid = Liquid {
        dimensions: [2, 2],
        ..SlotConstruct::construct(&mut slot_values)
    };
    assert!(liquid_mesh(&liquid, Vec2::from_array([1.; 2])).is_err());
}

#[test]
fn explicit_portal_reachability_is_conservative_and_keeps_other_systems() {
    let mut slot_values = SlotValues::default();
    let system = system_id();
    let other = system_id();
    let a = GroupId { system, index: 0 };
    let b = GroupId { system, index: 1 };
    let isolated = GroupId { system, index: 2 };
    let runtime = WorldRuntime {
        neighbors: HashMap::from([(a, vec![b]), (b, vec![a]), (isolated, vec![])]),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let selected = runtime.visibility(&WorldSelection {
        portals: WorldPortalPolicy::ReachableFrom(a),
        ..SlotConstruct::construct(&mut slot_values)
    });
    assert!(selected.group_visible(a));
    assert!(selected.group_visible(b));
    assert!(!selected.group_visible(isolated));
    assert!(selected.group_visible(GroupId {
        system: other,
        index: 2
    }));
    let all = runtime.visibility(&WorldSelection::default());
    assert!(all.group_visible(isolated));
    let unknown = runtime.visibility(&WorldSelection {
        portals: WorldPortalPolicy::ReachableFrom(GroupId { system, index: 99 }),
        ..SlotConstruct::construct(&mut slot_values)
    });
    assert!(unknown.group_visible(isolated));
}

#[test]
fn explicit_doodad_selection_keeps_common_set() {
    let mut slot_values = SlotValues::default();
    let system = system_id();
    let common = DoodadSetId { system, index: 0 };
    let chosen = DoodadSetId { system, index: 2 };
    let runtime = WorldRuntime::default();
    let selected = runtime.visibility(&WorldSelection {
        doodad_sets: Some(HashSet::from([chosen])),
        ..SlotConstruct::construct(&mut slot_values)
    });
    assert!(selected.set_visible(common));
    assert!(selected.set_visible(chosen));
    assert!(!selected.set_visible(DoodadSetId { system, index: 1 }));
}

#[test]
fn spherical_volume_fades_without_mutating_scene_lighting() {
    let volume = Volume::Sphere {
        center: Vec3::from_array([0.; 3]),
        radii: Vec2::from_array([2., 4.]),
    };
    assert_eq!(world_volume_weight(&volume, Vec3::ZERO), 1.);
    assert_eq!(world_volume_weight(&volume, Vec3::new(3., 0., 0.)), 0.5);
    assert_eq!(world_volume_weight(&volume, Vec3::new(5., 0., 0.)), 0.);
}

#[test]
fn compiled_world_spawns_liquid_hooks_and_selection_gates() {
    let mut slot_values = SlotValues::default();
    use super::super::{compile_model, spawn_model, ModelMaterial, PoseBaker, PoseOptions};
    use bevy::ecs::world::CommandQueue;
    use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
    use wc3::model::profiles::wmo::{Group, Region};
    use wc3::model::{GeometryData, Mesh as ModelMesh, MeshInstance};

    let mut builder = ModelBuilder::<Universal>::new(ModelMetadata::Wmo(Default::default()));
    let node = builder.add_named_node("region").unwrap();
    let geometry = GeometryData::triangles(
        ([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])
            .into_iter()
            .map(Vec3::from_array)
            .collect::<PackedArray<_>>(),
        [[0, 1, 2]],
    )
    .unwrap();
    let mesh = builder.add_mesh(ModelMesh::new(geometry)).unwrap();
    let mut placement = MeshInstance::new(node, mesh);
    placement.lod = Some(2);
    placement.properties = Some(InstanceProperties::Wmo(Box::new(Region {
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
            ..SlotConstruct::construct(&mut slot_values)
        }],
        ..SlotConstruct::construct(&mut slot_values)
    })));
    let instance = builder.add_instance(placement).unwrap();
    let system = builder
        .add_system(System::Wmo(WorldSystem {
            groups: vec![Group {
                instances: vec![instance],
                ..SlotConstruct::construct(&mut slot_values)
            }],
            ..SlotConstruct::construct(&mut slot_values)
        }))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    assert_eq!(compiled.world.groups.len(), 1);
    assert_eq!(compiled.liquids.len(), 1);
    assert!(compiled.liquids[0].surface.is_none());
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    assert_eq!(prepared.world_liquids()[0].liquid_type, 42);
    // Baking uses exact requested authored levels, including liquid-only levels.
    for (lod, expected) in [(None, true), (Some(0), false), (Some(2), true)] {
        let baked = PoseBaker::default()
            .bake(
                &prepared,
                &meshes,
                &binds,
                PoseOptions {
                    lod,
                    ..SlotConstruct::construct(&mut slot_values)
                },
            )
            .unwrap();
        assert_eq!(baked.parts.len(), 1);
        assert_eq!(baked.parts[0].visible, expected);
        assert_eq!(baked.bounds.is_some(), expected);
    }
    let mut materials = Assets::<ModelMaterial>::default();
    let mut app = App::new();
    let mut queue = CommandQueue::default();
    let root = {
        let mut commands = Commands::new(&mut queue, app.world());
        spawn_model(
            &mut commands,
            &mut meshes,
            &mut materials,
            &prepared,
            Transform::default(),
        )
    };
    queue.apply(app.world_mut());
    app.init_resource::<LodSettings>();
    app.add_systems(Startup, update_lod);
    app.add_systems(Update, apply_world_selection);
    app.world_mut().get_mut::<Playback>(root).unwrap().lod = 2;
    app.update();
    assert_eq!(app.world().get::<Playback>(root).unwrap().lod, 2);
    let liquid = app
        .world_mut()
        .query_filtered::<Entity, With<WorldLiquidSurface>>()
        .single(app.world())
        .unwrap();
    assert!(app
        .world()
        .get::<MeshMaterial3d<ModelMaterial>>(liquid)
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
        .get_mut::<WorldSelection>(root)
        .unwrap()
        .hidden_groups
        .insert(GroupId { system, index: 0 });
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(gate).unwrap(),
        Visibility::Hidden
    );
    app.world_mut()
        .get_mut::<WorldSelection>(root)
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
    app.world_mut().get_mut::<Playback>(root).unwrap().lod = 0;
    app.update();
    assert_eq!(
        *app.world().get::<Visibility>(gate).unwrap(),
        Visibility::Hidden
    );
}

#[test]
fn default_placement_selection_includes_common_without_combining_variants() {
    let mut slot_values = SlotValues::default();
    let system = system_id();
    let runtime = WorldRuntime::default();
    let common = DoodadSetId { system, index: 0 };
    let alternative = DoodadSetId { system, index: 1 };
    let selected = runtime.visibility(&WorldSelection::default());
    assert!(selected.set_visible(common));
    assert!(!selected.set_visible(alternative));
    let all = runtime.visibility(&WorldSelection {
        doodad_sets: None,
        ..SlotConstruct::construct(&mut slot_values)
    });
    assert!(all.set_visible(alternative));
}

fn portal_groups(system: SystemId, count: usize) -> WorldRuntime {
    let mut runtime = WorldRuntime {
        groups: (0..count)
            .map(|index| WorldGroup {
                id: GroupId {
                    system,
                    index: index as u32,
                },
                name: format!("room {index}"),
                instances: Arc::from([]),
                bounds: [Vec3::from_array([-100.0; 3]), Vec3::from_array([100.0; 3])],
                options: GroupOptions {
                    interior: true,
                    ..default()
                },
                detail_selection: None,
            })
            .collect::<Vec<_>>()
            .into(),
        ..default()
    };
    runtime.index_portal_groups();
    runtime
}

fn window(
    runtime: &mut WorldRuntime,
    system: SystemId,
    from: usize,
    to: usize,
    x: [f32; 2],
    z: f32,
) {
    let index = runtime.portals.len();
    let id = PortalId {
        system,
        index: index as u32,
    };
    let mut portals = runtime.portals.to_vec();
    portals.push(WorldPortal {
        id,
        vertices: Arc::from(
            [
                [x[0], -1.0, z],
                [x[1], -1.0, z],
                [x[1], 1.0, z],
                [x[0], 1.0, z],
            ]
            .map(Vec3::from_array),
        ),
        plane: Vec4::from_array([0.0, 0.0, 1.0, -z]),
        adjustment: None,
    });
    runtime.portals = portals.into();
    runtime.portal_indices.insert(id, index);
    let a = GroupId {
        system,
        index: from as u32,
    };
    let b = GroupId {
        system,
        index: to as u32,
    };
    runtime
        .portal_links
        .entry(a)
        .or_default()
        .push(PortalConnection {
            portal: id,
            neighbor: b,
            side: 1,
            skip_viewer: false,
            adjusted_plane: false,
        });
    runtime.neighbors.entry(a).or_default().push(b);
}

fn forward_view(system: SystemId) -> PortalView {
    PortalView::new(
        GroupId { system, index: 0 },
        Vec3::ZERO,
        [
            Vec4::new(1.0, 0.0, 1.0, 0.0),
            Vec4::new(-1.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 1.0, 1.0, 0.0),
            Vec4::new(0.0, -1.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, -0.01),
        ],
    )
    .unwrap()
}

#[test]
fn portal_view_validates_and_normalizes_halfspaces() {
    let group = GroupId {
        system: system_id(),
        index: 0,
    };
    assert!(PortalView::new(group, Vec3::NAN, []).is_err());
    assert!(PortalView::new(group, Vec3::ZERO, [Vec4::ZERO]).is_err());
    assert!(PortalView::new(group, Vec3::ZERO, [Vec4::NAN]).is_err());
    let view = PortalView::new(group, Vec3::ONE, [Vec4::new(2.0, 0.0, 0.0, -4.0)]).unwrap();
    assert_eq!(view.group(), group);
    assert_eq!(view.eye(), Vec3::ONE);
    assert_eq!(view.planes(), &[Vec4::new(1.0, 0.0, 0.0, -2.0)]);
}

#[test]
fn clipped_portals_test_frustum_then_narrow_neighbor_windows() {
    let system = system_id();
    let mut runtime = portal_groups(system, 4);
    window(&mut runtime, system, 0, 1, [-0.5, 0.5], 2.0);
    window(&mut runtime, system, 1, 2, [2.0, 3.0], 4.0);
    window(&mut runtime, system, 0, 3, [10.0, 11.0], 2.0);
    let selected = runtime.visibility(&WorldSelection {
        portals: WorldPortalPolicy::Clipped(forward_view(system)),
        ..default()
    });
    assert!(selected.group_visible(GroupId { system, index: 0 }));
    assert!(selected.group_visible(GroupId { system, index: 1 }));
    assert!(!selected.group_visible(GroupId { system, index: 2 }));
    assert!(!selected.group_visible(GroupId { system, index: 3 }));
}

#[test]
fn clipped_portals_revisit_groups_through_distinct_windows_and_stop_cycles() {
    let system = system_id();
    let mut runtime = portal_groups(system, 6);
    // Stack order encounters the left path first; only the later right path
    // exposes room4. A global visited-group set would incorrectly hide it.
    window(&mut runtime, system, 0, 2, [1.0, 2.0], 2.0);
    window(&mut runtime, system, 0, 1, [-2.0, -1.0], 2.0);
    window(&mut runtime, system, 1, 3, [-4.0, -2.0], 4.0);
    window(&mut runtime, system, 2, 3, [2.0, 4.0], 4.0);
    window(&mut runtime, system, 3, 4, [4.0, 6.0], 6.0);
    window(&mut runtime, system, 3, 5, [-0.2, 0.2], 6.0);
    window(&mut runtime, system, 3, 0, [-4.0, 4.0], 4.0);
    let selected = runtime.visibility(&WorldSelection {
        portals: WorldPortalPolicy::Clipped(forward_view(system)),
        ..default()
    });
    assert!((0..5).all(|index| selected.group_visible(GroupId { system, index })));
    assert!(!selected.group_visible(GroupId { system, index: 5 }));
}

#[test]
fn clipped_portals_keep_exterior_unknown_views_and_malformed_windows_conservative() {
    let system = system_id();
    let mut runtime = portal_groups(system, 3);
    let mut groups = runtime.groups.to_vec();
    groups[2].options.exterior = true;
    runtime.groups = groups.into();
    runtime.index_portal_groups();
    window(&mut runtime, system, 0, 1, [10.0, 11.0], 2.0);
    // An unresolved adjusted-plane instruction cannot safely reject its neighbor.
    runtime
        .portal_links
        .get_mut(&GroupId { system, index: 0 })
        .unwrap()[0]
        .adjusted_plane = true;
    let mut selection = WorldSelection {
        portals: WorldPortalPolicy::Clipped(forward_view(system)),
        ..default()
    };
    let selected = runtime.visibility(&selection);
    assert!(selected.group_visible(GroupId { system, index: 1 }));
    assert!(selected.group_visible(GroupId { system, index: 2 }));
    assert!(selected.group_visible(GroupId {
        system: system_id(),
        index: 99
    }));
    selection.portals = WorldPortalPolicy::Clipped(
        PortalView::new(GroupId { system, index: 99 }, Vec3::ZERO, []).unwrap(),
    );
    assert!(runtime
        .visibility(&selection)
        .group_visible(GroupId { system, index: 1 }));
    selection.portals = WorldPortalPolicy::Clipped(
        PortalView::new(GroupId { system, index: 2 }, Vec3::ZERO, []).unwrap(),
    );
    assert!(runtime
        .visibility(&selection)
        .group_visible(GroupId { system, index: 1 }));
    assert!(portal_window(&[Vec3::ZERO; 2], Vec3::ZERO, &[]).is_none());
    assert!(portal_window(
        &[
            Vec3::from_array([0.0, 0.0, 2.0]),
            Vec3::from_array([1.0, 0.0, 2.0]),
            Vec3::from_array([0.0, 1.0, 3.0]),
            Vec3::from_array([1.0, 1.0, 2.0])
        ],
        Vec3::ZERO,
        &[]
    )
    .is_none());
}

#[test]
fn portal_traversal_budget_falls_back_to_all_groups() {
    let system = system_id();
    let mut runtime = portal_groups(system, 9);
    // Missing portal geometry remains open. Eight mutually connected rooms have
    // more simple paths than the work budget, despite their tiny authored graph.
    for from in 0..8 {
        for to in 0..8 {
            if from == to {
                continue;
            }
            runtime
                .portal_links
                .entry(GroupId {
                    system,
                    index: from,
                })
                .or_default()
                .push(PortalConnection {
                    portal: PortalId { system, index: 99 },
                    neighbor: GroupId { system, index: to },
                    side: 0,
                    skip_viewer: false,
                    adjusted_plane: false,
                });
        }
    }
    let selected = runtime.visibility(&WorldSelection {
        portals: WorldPortalPolicy::Clipped(forward_view(system)),
        ..default()
    });
    assert!(selected.group_visible(GroupId { system, index: 8 }));
}

#[test]
fn portal_boundaries_and_degenerate_windows_remain_open() {
    let polygon = [
        [-1.0, -1.0, 2.0],
        [1.0, -1.0, 2.0],
        [1.0, 1.0, 2.0],
        [-1.0, 1.0, 2.0],
    ];
    assert!(portal_window(
        &polygon.map(Vec3::from_array),
        Vec3::new(0.0, 0.0, 2.0),
        &[]
    )
    .is_none());
    let parent = [Vec4::new(0.0, 0.0, 1.0, -0.1)];
    assert_eq!(
        window_planes(&[Vec3::new(0.0, 0.0, 2.0)], Vec3::ZERO, &parent),
        parent
    );
    assert_eq!(
        window_planes(
            &[
                Vec3::new(0.0, 0.0, 2.0),
                Vec3::new(1.0, 0.0, 2.0),
                Vec3::new(2.0, 0.0, 2.0)
            ],
            Vec3::ZERO,
            &parent
        ),
        parent
    );
    let group = GroupId {
        system: system_id(),
        index: 0,
    };
    assert!(PortalView::new(group, Vec3::ZERO, [Vec4::new(0.001, 0.0, 0.0, f32::MAX)]).is_err());
    // Finite inputs can overflow intermediate dot products; such geometry is
    // ambiguous and must not become a rejected/empty window.
    assert!(portal_window(
        &polygon.map(Vec3::from_array),
        Vec3::ZERO,
        &[Vec4::splat(f32::MAX)]
    )
    .is_none());
}
