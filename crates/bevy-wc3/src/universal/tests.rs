use super::*;
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::mesh::VertexAttributeValues;
use wc3::model::materials::layered::{Layer, LayeredSurface};
use wc3::model::{
    Animated, Draw, ExplicitBinding, GeometryData, GeometryInfluences, IdentityBinding, Influence,
    Material as ModelMaterial, MaterialSurface, Mesh as ModelMesh, MeshInstance, Node, Skin,
    UniversalBinding, ZeroVec,
};
use wc3::model::{Axis as ModelAxis, Handedness, LengthUnit, Model};

fn fixture(binding: UniversalBinding<'static>) -> (Model<'static>, NodeId) {
    fixture_geometry(
        GeometryData::triangles([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], [[0, 1, 2]]).unwrap(),
        binding,
    )
}
fn fixture_geometry<'a>(
    mut geometry: GeometryData<'a>,
    binding: UniversalBinding<'a>,
) -> (Model<'a>, NodeId) {
    let mut builder = Model::builder("explicit skin fixture");
    let joint = builder
        .add_node(
            Node::named("joint")
                .with_transform(ModelTransform {
                    translation: [3., 0., 0.],
                    ..Default::default()
                })
                .unwrap(),
        )
        .unwrap();
    let mount = builder
        .add_node(
            Node::named("mesh placement")
                .with_transform(ModelTransform {
                    translation: [7., 2., 0.],
                    ..Default::default()
                })
                .unwrap(),
        )
        .unwrap();
    let skin = builder.add_skin(Skin::new(vec![joint], binding)).unwrap();
    geometry.influences = GeometryInfluences::Four(
        [[
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
        ]; 3]
            .into_iter()
            .collect(),
    );
    let mut mesh = ModelMesh::new(geometry);
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let material = builder
        .add_material(ModelMaterial {
            name: "plain".into(),
            surface: MaterialSurface::Layered(Box::new(LayeredSurface {
                layers: vec![Layer {
                    opacity: Animated::constant(1.),
                    ..Default::default()
                }],
                ..Default::default()
            })),
        })
        .unwrap();
    let mut instance = MeshInstance::new(mount, mesh);
    instance.skin = Some(skin);
    instance.draws.push(Draw::new(subset, Default::default()));
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    (builder.finish().unwrap(), joint)
}
#[test]
fn explicit_and_identity_bindings_preserve_authored_matrices() {
    let explicit = Mat4::from_translation(Vec3::new(-3., 0., 0.));
    let (source, _) = fixture(UniversalBinding::Explicit(
        ExplicitBinding::new([explicit.to_cols_array_2d()]).unwrap(),
    ));
    let compiled = compile_universal_model(&source, |_| None).unwrap();
    assert_eq!(compiled.skins[0].binds[0], explicit);
    let (identity, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let identity = compile_universal_model(&identity, |_| None).unwrap();
    assert_eq!(identity.skins[0].binds[0], Mat4::IDENTITY);
    assert_eq!(compiled.draws.len(), 1);
}
#[test]
fn transformed_mesh_placement_cancels_in_joint_skin_matrix() {
    let inverse = Mat4::from_translation(Vec3::new(-3., 0., 0.));
    let (source, joint_id) = fixture(UniversalBinding::Explicit(
        ExplicitBinding::new([inverse.to_cols_array_2d()]).unwrap(),
    ));
    let compiled = compile_universal_model(&source, |_| None).unwrap();
    drop(source);
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut app = App::new();
    app.add_plugins(TransformPlugin);
    let mut materials = Assets::default();
    let mut queue = CommandQueue::default();
    let root = spawn_universal_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::from_translation(Vec3::new(10., 0., 0.)),
    );
    queue.apply(app.world_mut());
    app.update();
    let joint = app
        .world()
        .get::<UniversalInstance>(root)
        .unwrap()
        .node(joint_id)
        .unwrap();
    let joint_world = app
        .world()
        .get::<GlobalTransform>(joint)
        .unwrap()
        .to_matrix();
    let mut draws = app.world_mut().query::<(&SkinnedMesh, &GlobalTransform)>();
    let (skin, mesh_world) = draws.single(app.world()).unwrap();
    assert_eq!(skin.joints, vec![joint]);
    let bind = binds.get(&skin.inverse_bindposes).unwrap()[0];
    let vertex = Vec3::new(1., 0., 0.);
    let local_skin = mesh_world.to_matrix().inverse() * joint_world * bind;
    let actual = mesh_world
        .to_matrix()
        .transform_point3(local_skin.transform_point3(vertex));
    let expected = (joint_world * inverse).transform_point3(vertex);
    assert!((actual - expected).length() < 1e-5);
    assert!(app
        .world()
        .get::<UniversalAnimationError>(root)
        .unwrap()
        .0
        .is_none());
}
#[test]
fn compiled_mesh_survives_borrowed_position_storage() {
    let compiled = {
        let packed: ZeroVec<'_, [f32; 3]> = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]
            .into_iter()
            .collect();
        let bytes = packed.as_bytes().to_vec();
        let borrowed = ZeroVec::parse_bytes(&bytes).unwrap();
        let geometry = GeometryData::triangles(borrowed, [[0, 1, 2]]).unwrap();
        let (graph, _) = fixture_geometry(geometry, UniversalBinding::Identity(IdentityBinding));
        compile_universal_model(&graph, |_| None).unwrap()
    };
    assert_eq!(compiled.draws.len(), 1);
    let Some(VertexAttributeValues::Float32x3(values)) =
        compiled.draws[0].mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("positions absent")
    };
    assert_eq!(values, &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
}

fn assert_matrix_close(actual: Mat4, expected: Mat4) {
    for (actual, expected) in actual
        .to_cols_array()
        .into_iter()
        .zip(expected.to_cols_array())
    {
        assert!(
            (actual - expected).abs() < 2e-5,
            "matrix component {actual} != {expected}"
        );
    }
}
#[test]
fn rendered_hierarchy_preserves_shear_and_declared_coordinate_units() {
    use wc3::model::{Basis, Quaternion};
    let (mut source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let mount = source.instances().next().unwrap().1.node;
    let parent_rotation = Quat::from_rotation_z(0.3);
    let child_rotation = Quat::from_rotation_z(0.7);
    source
        .edit(|builder| {
            builder.set_basis(Basis::new(
                ModelAxis::PositiveZ,
                ModelAxis::PositiveY,
                Handedness::Right,
            )?);
            builder.set_units(LengthUnit::MetersPerUnit(0.25));
            let parent = builder.add_node(Node::named("nonuniform parent").with_transform(
                ModelTransform {
                    translation: [1., 2., 3.],
                    rotation: Quaternion(parent_rotation.to_array()),
                    scale: [2., 1., 0.5],
                    ..Default::default()
                },
            )?)?;
            builder.set_node(
                mount,
                Node::named("rotated child")
                    .with_parent(parent)
                    .with_transform(ModelTransform {
                        translation: [4., 0., 0.],
                        rotation: Quaternion(child_rotation.to_array()),
                        ..Default::default()
                    })?,
            )
        })
        .unwrap();
    let compiled = compile_universal_model(&source, |_| None).unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let root_transform = Transform::from_translation(Vec3::new(10., 20., 30.))
        .with_rotation(Quat::from_rotation_y(0.2));
    let mut app = App::new();
    app.add_plugins(TransformPlugin);
    app.add_systems(Update, animate_universal_instances);
    let mut materials = Assets::default();
    let mut queue = CommandQueue::default();
    let root = spawn_universal_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
        root_transform,
    );
    queue.apply(app.world_mut());
    app.insert_resource(materials);
    app.update();
    let mount = app
        .world()
        .get::<UniversalInstance>(root)
        .unwrap()
        .node(mount)
        .unwrap();
    let draw = app
        .world_mut()
        .query::<(Entity, &UniversalDraw, &ChildOf)>()
        .iter(app.world())
        .find(|(_, _, parent)| parent.parent() == mount)
        .unwrap()
        .0;
    let basis = Mat4::from_cols(
        Vec3::X.extend(0.),
        Vec3::NEG_Z.extend(0.),
        Vec3::Y.extend(0.),
        Vec4::W,
    ) * Mat4::from_scale(Vec3::splat(0.25));
    let parent = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 1., 0.5),
        parent_rotation,
        Vec3::new(1., 2., 3.),
    );
    let child = Mat4::from_rotation_translation(child_rotation, Vec3::new(4., 0., 0.));
    let expected = root_transform.to_matrix() * basis * parent * child;
    assert!(
        expected
            .x_axis
            .truncate()
            .dot(expected.y_axis.truncate())
            .abs()
            > 0.01,
        "fixture must contain real shear"
    );
    assert_matrix_close(
        app.world()
            .get::<GlobalTransform>(mount)
            .unwrap()
            .to_matrix(),
        expected,
    );
    assert_matrix_close(
        app.world()
            .get::<GlobalTransform>(draw)
            .unwrap()
            .to_matrix(),
        expected,
    );
}
#[test]
fn asset_population_retains_existing_root_controls_and_source_identity() {
    use super::instances::{spawn_loaded_universal_models, UniversalPreparedCache};
    let (source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let compiled = compile_universal_model(&source, |_| None).unwrap();
    let mut app = App::new();
    app.init_resource::<Assets<UniversalModelAsset>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<SkinnedMeshInverseBindposes>>()
        .init_resource::<Assets<UniversalMaterial>>()
        .init_resource::<UniversalPreparedCache>();
    app.add_systems(Update, spawn_loaded_universal_models);
    let handle = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(UniversalModelAsset {
            compiled,
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        });
    let source_id = handle.id();
    let transform = Transform::from_xyz(5., 6., 7.).with_scale(Vec3::new(2., 3., 4.));
    let root = app
        .world_mut()
        .spawn((
            UniversalModelInstance(handle),
            transform,
            UniversalPlayback {
                clip_time: 0.42,
                timeline_time: 1.25,
                shared_time: 3.5,
                speed: 0.37,
                paused: true,
                lod: 7,
                ..Default::default()
            },
        ))
        .id();
    app.update();
    assert!(app.world().get::<UniversalModelInstance>(root).is_none());
    assert!(app.world().get::<UniversalInstance>(root).is_some());
    assert_eq!(
        app.world().get::<UniversalModelSource>(root).unwrap().0,
        source_id
    );
    assert_eq!(*app.world().get::<Transform>(root).unwrap(), transform);
    let playback = app.world().get::<UniversalPlayback>(root).unwrap();
    assert_eq!(playback.clip_time, 0.42);
    assert_eq!(playback.timeline_time, 1.25);
    assert_eq!(playback.shared_time, 3.5);
    assert_eq!(playback.speed, 0.37);
    assert!(playback.paused);
    assert_eq!(playback.lod, 7);
}

#[test]
fn replacing_asset_population_removes_only_the_owned_old_hierarchy() {
    use super::instances::{spawn_loaded_universal_models, UniversalPreparedCache};
    let (source, joint) = fixture(UniversalBinding::Identity(IdentityBinding));
    let compiled = compile_universal_model(&source, |_| None).unwrap();
    let mut app = App::new();
    app.init_resource::<Assets<UniversalModelAsset>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<SkinnedMeshInverseBindposes>>()
        .init_resource::<Assets<UniversalMaterial>>()
        .init_resource::<UniversalPreparedCache>();
    app.add_systems(Update, spawn_loaded_universal_models);
    let first = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(UniversalModelAsset {
            compiled: compiled.clone(),
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        });
    let second = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(UniversalModelAsset {
            compiled,
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        });
    let override_pose = ModelTransform {
        translation: [9., 8., 7.],
        ..Default::default()
    };
    let root = app
        .world_mut()
        .spawn((
            UniversalModelInstance(first),
            UniversalPlayback {
                clip_time: 0.42,
                paused: true,
                ..Default::default()
            },
            UniversalPoseOverrides(HashMap::from([(joint, override_pose)])),
        ))
        .id();
    let application_child = app
        .world_mut()
        .spawn((Name::new("application child"), ChildOf(root)))
        .id();
    app.update();
    let old_node = app
        .world()
        .get::<UniversalInstance>(root)
        .unwrap()
        .node(joint)
        .unwrap();
    let old_basis = app.world().get::<ChildOf>(old_node).unwrap().parent();
    let old_draws: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<UniversalDraw>>()
        .iter(app.world())
        .collect();
    app.world_mut()
        .entity_mut(root)
        .insert(UniversalModelInstance(second));
    app.update();
    assert!(app.world().get_entity(old_node).is_err());
    assert!(app.world().get_entity(old_basis).is_err());
    for draw in old_draws {
        assert!(app.world().get_entity(draw).is_err());
    }
    assert_eq!(
        app.world()
            .get::<ChildOf>(application_child)
            .unwrap()
            .parent(),
        root
    );
    assert_eq!(
        app.world()
            .get::<UniversalPlayback>(root)
            .unwrap()
            .clip_time,
        0.42
    );
    assert!(app.world().get::<UniversalPlayback>(root).unwrap().paused);
    assert_eq!(
        app.world().get::<UniversalPoseOverrides>(root).unwrap().0[&joint],
        override_pose
    );
    assert_ne!(
        app.world()
            .get::<UniversalInstance>(root)
            .unwrap()
            .node(joint)
            .unwrap(),
        old_node
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<UniversalDraw>>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(app.world().get::<Children>(root).unwrap().len(), 2);
}

fn material_handle(app: &mut App, root: Entity, node: NodeId) -> Handle<UniversalMaterial> {
    let node = app
        .world()
        .get::<UniversalInstance>(root)
        .unwrap()
        .node(node)
        .unwrap();
    app.world_mut()
        .query::<(&MeshMaterial3d<UniversalMaterial>, &ChildOf)>()
        .iter(app.world())
        .find(|(_, parent)| parent.parent() == node)
        .unwrap()
        .0
         .0
        .clone()
}
fn spawn_pair(source: &Model<'_>) -> (App, Entity, Entity, NodeId) {
    let node = source.instances().next().unwrap().1.node;
    let compiled = compile_universal_model(source, |_| None).unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut app = App::new();
    app.add_plugins(TransformPlugin);
    app.add_systems(Update, animate_universal_instances);
    let mut materials = Assets::default();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let first = spawn_universal_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    let second = spawn_universal_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::from_xyz(10., 0., 0.),
    );
    queue.apply(app.world_mut());
    app.insert_resource(materials);
    (app, first, second, node)
}
#[test]
fn constant_materials_share_assets_between_instances() {
    let (source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let (mut app, first, second, node) = spawn_pair(&source);
    assert_eq!(
        material_handle(&mut app, first, node),
        material_handle(&mut app, second, node)
    );
}
#[test]
fn animated_materials_are_isolated_between_playback_clocks() {
    use wc3::model::profiles::mdlx::properties::Layer as LayerProperty;
    use wc3::model::{Channel, Clock, Curve, Time as ModelTime, Track};
    let (mut source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let material = source.materials().next().unwrap().0;
    let property = LayerProperty(0).opacity().bind(material);
    source
        .edit(|builder| {
            builder.set_material_surface(
                material,
                MaterialSurface::Layered(Box::new(LayeredSurface {
                    layers: vec![Layer {
                        opacity: Animated::new(1., property.clone()),
                        ..Default::default()
                    }],
                    ..Default::default()
                })),
            )?;
            builder.add_timeline_channel(Channel::Scalar(Track {
                target: property.widen(),
                curve: Curve::linear(
                    [ModelTime::seconds(0.)?, ModelTime::seconds(1.)?],
                    [1.0f32, 0.],
                )?,
                clock: Clock::Timeline,
            }))
        })
        .unwrap();
    let (mut app, first, second, node) = spawn_pair(&source);
    app.world_mut()
        .get_mut::<UniversalPlayback>(first)
        .unwrap()
        .timeline_time = 0.25;
    app.world_mut()
        .get_mut::<UniversalPlayback>(second)
        .unwrap()
        .timeline_time = 0.75;
    app.update();
    let first_material = material_handle(&mut app, first, node);
    let second_material = material_handle(&mut app, second, node);
    assert_ne!(first_material, second_material);
    let materials = app.world().resource::<Assets<UniversalMaterial>>();
    assert!(
        (materials
            .get(&first_material)
            .unwrap()
            .base
            .base_color
            .alpha()
            - 0.75)
            .abs()
            < 1e-5
    );
    assert!(
        (materials
            .get(&second_material)
            .unwrap()
            .base
            .base_color
            .alpha()
            - 0.25)
            .abs()
            < 1e-5
    );
}
