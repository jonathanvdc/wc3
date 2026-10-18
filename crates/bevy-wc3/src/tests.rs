use super::*;
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::materials::mdlx::{MaterialInputs, MaterialStep as MdlxStep, Shader};
use wc3::model::profiles::mdlx;
use wc3::model::profiles::mdlx::Material as MdlxMaterial;
use wc3::model::Model;
use wc3::model::ModelBuilder;
use wc3::model::ModelMetadata;
use wc3::model::{
    Billboard, Draw, ExplicitBinding, GeometryData, GeometryInfluences, IdentityBinding, Influence,
    Material as SemanticMaterial, MaterialSurface, Mesh as ModelMesh, MeshInstance, Node, Skin,
    UniversalBinding, ZeroVec,
};
use wc3::model::{BillboardFrame, BillboardScale};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{NodeId, Transform as ModelTransform};

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
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(mdlx::ModelMetadata {
        bounds: None,
        ..Default::default()
    }));
    let joint = builder
        .add_node(
            Node::named("joint", &mut slot_values)
                .with_transform(
                    ModelTransform {
                        translation: [3., 0., 0.],
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    &mut slot_values,
                )
                .unwrap(),
        )
        .unwrap();
    let mount = builder
        .add_node(
            Node::named("mesh placement", &mut slot_values)
                .with_transform(
                    ModelTransform {
                        translation: [7., 2., 0.],
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    &mut slot_values,
                )
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
        .add_material(SemanticMaterial {
            name: "plain".into(),
            surface: MaterialSurface::from(MdlxMaterial {
                steps: vec![MdlxStep {
                    shader: Shader::Classic(MaterialInputs {
                        opacity: slot_values.add(1.),
                        ..SlotConstruct::construct(&mut slot_values)
                    }),
                    ..SlotConstruct::construct(&mut slot_values)
                }],
            }),
        })
        .unwrap();
    let mut instance = MeshInstance::new(mount, mesh);
    instance.skin = Some(skin);
    instance.draws.push(Draw::new(
        subset,
        SlotConstruct::construct(&mut slot_values),
    ));
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    (
        {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap(),
        joint,
    )
}
#[test]
fn explicit_and_identity_bindings_preserve_authored_matrices() {
    let explicit = Mat4::from_translation(Vec3::new(-3., 0., 0.));
    let (source, _) = fixture(UniversalBinding::Explicit(
        ExplicitBinding::new([explicit.to_cols_array_2d()]).unwrap(),
    ));
    let compiled = compile_model(&source, |_| None).unwrap();
    assert_eq!(compiled.skins[0].binds[0], explicit);
    let (identity, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let identity = compile_model(&identity, |_| None).unwrap();
    assert_eq!(identity.skins[0].binds[0], Mat4::IDENTITY);
    assert_eq!(compiled.draws.len(), 1);
}
#[test]
fn transformed_mesh_placement_cancels_in_joint_skin_matrix() {
    let inverse = Mat4::from_translation(Vec3::new(-3., 0., 0.));
    let (source, joint_id) = fixture(UniversalBinding::Explicit(
        ExplicitBinding::new([inverse.to_cols_array_2d()]).unwrap(),
    ));
    let compiled = compile_model(&source, |_| None).unwrap();
    drop(source);
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut app = App::new();
    app.add_plugins(TransformPlugin);
    let mut materials = Assets::default();
    let mut queue = CommandQueue::default();
    let root = spawn_model(
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
        .get::<ModelRig>(root)
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
    assert!(app.world().get::<PlaybackError>(root).unwrap().0.is_none());
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
        compile_model(&graph, |_| None).unwrap()
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
fn rendered_hierarchy_preserves_shear() {
    let mut slot_values = SlotValues::default();
    use wc3::model::Quaternion;
    let (mut source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let mount = source.instances().iter().next().unwrap().1.node;
    let parent_rotation = Quat::from_rotation_z(0.3);
    let child_rotation = Quat::from_rotation_z(0.7);
    source
        .edit(|builder| {
            let edit_result = (|| {
                let parent = builder.add_node(
                    Node::named("nonuniform parent", &mut slot_values).with_transform(
                        ModelTransform {
                            translation: [1., 2., 3.],
                            rotation: Quaternion(parent_rotation.to_array()),
                            scale: [2., 1., 0.5],
                            ..SlotConstruct::construct(&mut slot_values)
                        },
                        &mut slot_values,
                    )?,
                )?;
                builder.set_node(
                    mount,
                    Node::named("rotated child", &mut slot_values)
                        .with_parent(parent)
                        .with_transform(
                            ModelTransform {
                                translation: [4., 0., 0.],
                                rotation: Quaternion(child_rotation.to_array()),
                                ..SlotConstruct::construct(&mut slot_values)
                            },
                            &mut slot_values,
                        )?,
                )
            })();
            builder.import_values(&slot_values)?;
            edit_result
        })
        .unwrap();
    let compiled = compile_model(&source, |_| None).unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let root_transform = Transform::from_translation(Vec3::new(10., 20., 30.))
        .with_rotation(Quat::from_rotation_y(0.2));
    let mut app = App::new();
    app.add_plugins(TransformPlugin);
    app.add_systems(Update, animate_instances);
    let mut materials = Assets::default();
    let mut queue = CommandQueue::default();
    let root = spawn_model(
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
        .get::<ModelRig>(root)
        .unwrap()
        .node(mount)
        .unwrap();
    let draw = app
        .world_mut()
        .query::<(Entity, &ModelDraw, &ChildOf)>()
        .iter(app.world())
        .find(|(_, _, parent)| parent.parent() == mount)
        .unwrap()
        .0;
    let basis = Mat4::from_cols(
        Vec3::X.extend(0.),
        Vec3::NEG_Z.extend(0.),
        Vec3::Y.extend(0.),
        Vec4::W,
    );
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
    let mut slot_values = SlotValues::default();
    use super::instances::{spawn_loaded_models, PreparedModelCache};
    let (source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let compiled = compile_model(&source, |_| None).unwrap();
    let mut app = App::new();
    app.init_resource::<Assets<ModelAsset>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<SkinnedMeshInverseBindposes>>()
        .init_resource::<Assets<ModelMaterial>>()
        .init_resource::<PreparedModelCache>();
    app.add_systems(Update, spawn_loaded_models);
    let handle = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .add(ModelAsset {
            compiled,
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        });
    let source_id = handle.id();
    let transform = Transform::from_xyz(5., 6., 7.).with_scale(Vec3::new(2., 3., 4.));
    let root = app
        .world_mut()
        .spawn((
            ModelInstance(handle),
            transform,
            Playback {
                clip_time: 0.42,
                shared_time: 3.5,
                speed: 0.37,
                paused: true,
                lod: 7,
                ..SlotConstruct::construct(&mut slot_values)
            },
        ))
        .id();
    app.update();
    assert!(app.world().get::<ModelInstance>(root).is_none());
    assert!(app.world().get::<ModelRig>(root).is_some());
    assert_eq!(
        app.world().get::<ModelSource>(root).unwrap().0.id(),
        source_id
    );
    assert_eq!(*app.world().get::<Transform>(root).unwrap(), transform);
    let playback = app.world().get::<Playback>(root).unwrap();
    assert_eq!(playback.clip_time, 0.42);
    assert_eq!(playback.shared_time, 3.5);
    assert_eq!(playback.speed, 0.37);
    assert!(playback.paused);
    assert_eq!(playback.lod, 7);
}

#[test]
fn replacing_asset_population_removes_only_the_owned_old_hierarchy() {
    let mut slot_values = SlotValues::default();
    use super::instances::{spawn_loaded_models, PreparedModelCache};
    let (source, joint) = fixture(UniversalBinding::Identity(IdentityBinding));
    let compiled = compile_model(&source, |_| None).unwrap();
    let mut app = App::new();
    app.init_resource::<Assets<ModelAsset>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<SkinnedMeshInverseBindposes>>()
        .init_resource::<Assets<ModelMaterial>>()
        .init_resource::<PreparedModelCache>();
    app.add_systems(Update, spawn_loaded_models);
    let first = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .add(ModelAsset {
            compiled: compiled.clone(),
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        });
    let second = app
        .world_mut()
        .resource_mut::<Assets<ModelAsset>>()
        .add(ModelAsset {
            compiled,
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        });
    let override_pose = ModelTransform {
        translation: [9., 8., 7.],
        ..SlotConstruct::construct(&mut slot_values)
    };
    let root = app
        .world_mut()
        .spawn((
            ModelInstance(first),
            Playback {
                clip_time: 0.42,
                paused: true,
                ..SlotConstruct::construct(&mut slot_values)
            },
            PoseOverrides(HashMap::from([(joint, override_pose)])),
        ))
        .id();
    let application_child = app
        .world_mut()
        .spawn((Name::new("application child"), ChildOf(root)))
        .id();
    app.update();
    let old_node = app
        .world()
        .get::<ModelRig>(root)
        .unwrap()
        .node(joint)
        .unwrap();
    let old_basis = app.world().get::<ChildOf>(old_node).unwrap().parent();
    let old_draws: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<ModelDraw>>()
        .iter(app.world())
        .collect();
    app.world_mut()
        .entity_mut(root)
        .insert(ModelInstance(second));
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
    assert_eq!(app.world().get::<Playback>(root).unwrap().clip_time, 0.42);
    assert!(app.world().get::<Playback>(root).unwrap().paused);
    assert_eq!(
        app.world().get::<PoseOverrides>(root).unwrap().0[&joint],
        override_pose
    );
    assert_ne!(
        app.world()
            .get::<ModelRig>(root)
            .unwrap()
            .node(joint)
            .unwrap(),
        old_node
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<ModelDraw>>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(app.world().get::<Children>(root).unwrap().len(), 2);
}

fn material_handle(app: &mut App, root: Entity, node: NodeId) -> Handle<ModelMaterial> {
    let node = app
        .world()
        .get::<ModelRig>(root)
        .unwrap()
        .node(node)
        .unwrap();
    app.world_mut()
        .query::<(&MeshMaterial3d<ModelMaterial>, &ChildOf)>()
        .iter(app.world())
        .find(|(_, parent)| parent.parent() == node)
        .unwrap()
        .0
         .0
        .clone()
}
fn spawn_pair(source: &Model<'_>) -> (App, Entity, Entity, NodeId) {
    let node = source.instances().iter().next().unwrap().1.node;
    let compiled = compile_model(source, |_| None).unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut app = App::new();
    app.add_plugins(TransformPlugin);
    app.add_systems(Update, animate_instances);
    let mut materials = Assets::default();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let first = spawn_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    let second = spawn_model(
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
    let mut slot_values = SlotValues::default();

    use wc3::model::{Curve, DynamicTrack, Time as ModelTime, TimeSource, Track};
    let (mut source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let material = source.materials().iter().next().unwrap().0;
    let property = slot_values.add(1.);
    source
        .edit(|builder| {
            let edit_result = (|| {
                builder.set_material_surface(
                    material,
                    MaterialSurface::from(MdlxMaterial {
                        steps: vec![MdlxStep {
                            shader: Shader::Classic(MaterialInputs {
                                opacity: property,
                                ..SlotConstruct::construct(&mut slot_values)
                            }),
                            ..SlotConstruct::construct(&mut slot_values)
                        }],
                    }),
                )?;
                builder.add_global_track(DynamicTrack::Scalar(Track {
                    target: property,
                    curve: Curve::linear(
                        [ModelTime::seconds(0.)?, ModelTime::seconds(1.)?],
                        [1.0f32, 0.],
                    )?,
                    clock: TimeSource::Animation,
                }))
            })();
            builder.import_values(&slot_values)?;
            edit_result
        })
        .unwrap();
    let (mut app, first, second, node) = spawn_pair(&source);
    app.world_mut()
        .get_mut::<Playback>(first)
        .unwrap()
        .clip_time = 0.25;
    app.world_mut()
        .get_mut::<Playback>(second)
        .unwrap()
        .clip_time = 0.75;
    app.update();
    let first_material = material_handle(&mut app, first, node);
    let second_material = material_handle(&mut app, second, node);
    assert_ne!(first_material, second_material);
    let materials = app.world().resource::<Assets<ModelMaterial>>();
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

#[test]
fn unchanged_static_instances_skip_sampling_but_external_alpha_still_updates_visibility() {
    let (source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let (mut app, first, _, node) = spawn_pair(&source);
    app.update();
    let handle = material_handle(&mut app, first, node);
    let initial = app
        .world()
        .get::<ModelRig>(first)
        .unwrap()
        .evaluation_count();
    for time in [1., 2., 3.] {
        {
            let mut playback = app.world_mut().get_mut::<Playback>(first).unwrap();
            playback.shared_time = time;
            playback.clip_time = time;
        }
        app.update();
    }
    assert_eq!(
        app.world()
            .get::<ModelRig>(first)
            .unwrap()
            .evaluation_count(),
        initial
    );
    app.world_mut()
        .resource_mut::<Assets<ModelMaterial>>()
        .get_mut(&handle)
        .unwrap()
        .base
        .base_color = Color::srgba(1., 1., 1., 0.);
    app.update();
    assert_eq!(
        app.world()
            .get::<ModelRig>(first)
            .unwrap()
            .evaluation_count(),
        initial
    );
    let draw_visibility = app
        .world_mut()
        .query::<(&MeshMaterial3d<ModelMaterial>, &Visibility)>()
        .iter(app.world())
        .find(|(material, _)| material.0 == handle)
        .unwrap()
        .1;
    assert_eq!(*draw_visibility, Visibility::Hidden);
}

#[test]
fn light_settings_insertion_and_removal_invalidate_static_evaluation() {
    let (source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let (mut app, first, _, _) = spawn_pair(&source);
    app.update();
    let initial = app
        .world()
        .get::<ModelRig>(first)
        .unwrap()
        .evaluation_count();
    app.world_mut()
        .entity_mut(first)
        .insert(ModelLightSettings::default());
    app.update();
    assert_eq!(
        app.world()
            .get::<ModelRig>(first)
            .unwrap()
            .evaluation_count(),
        initial + 1
    );
    app.world_mut()
        .entity_mut(first)
        .remove::<ModelLightSettings>();
    app.update();
    assert_eq!(
        app.world()
            .get::<ModelRig>(first)
            .unwrap()
            .evaluation_count(),
        initial + 2
    );
    app.update();
    assert_eq!(
        app.world()
            .get::<ModelRig>(first)
            .unwrap()
            .evaluation_count(),
        initial + 2
    );
}

#[test]
fn public_rest_bounds_use_joint_rest_pose_instead_of_mesh_placement() {
    let (source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let compiled = compile_model(&source, |_| None).unwrap();
    let bounds = compiled.rest_bounds().unwrap();
    let basis = compiled.basis.to_matrix();
    let points = [[3., 0., 0.], [4., 0., 0.], [3., 1., 0.]]
        .map(|point| basis.transform_point3(Vec3::from_array(point)));
    let min = points
        .into_iter()
        .fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let max = points
        .into_iter()
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    assert!((Vec3::from(bounds.center) - (min + max) * 0.5).length() < 1e-5);
    assert!((Vec3::from(bounds.half_extents) - (max - min) * 0.5).length() < 1e-5);
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    assert_eq!(prepared.rest_bounds().unwrap().center, bounds.center);
}

#[test]
fn m2_nonzero_transition_pairs_require_explicit_application_duration() {
    let mut slot_values = SlotValues::default();
    use wc3::model::controls::model::SequenceControls;
    use wc3::model::{Clip, ClipControls};
    let (mut source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    source
        .edit(|builder| {
            let edit_result = {
                let mut clip = Clip::named("M2 authored transition");
                clip.controls = Some(ClipControls::M2(SequenceControls {
                    blend_time: [150, 300],
                    ..SlotConstruct::construct(&mut slot_values)
                }));
                builder.add_clip(clip)
            };
            builder.import_values(&slot_values)?;
            edit_result
        })
        .unwrap();
    let compiled = compile_model(&source, |_| None).unwrap();
    assert!(compiled.diagnostics().iter().any(|diagnostic| {
        diagnostic.message.contains("[150, 300] milliseconds")
            && diagnostic.message.contains("play_with_blend")
    }));
}

#[test]
fn driving_camera_and_root_local_changes_update_billboards_before_propagation() {
    let (mut source, _) = fixture(UniversalBinding::Identity(IdentityBinding));
    let mount = source.instances().iter().next().unwrap().1.node;
    let mut node = source.node(mount).unwrap().clone();
    node.billboard = Billboard::Spherical {
        frame: BillboardFrame::Camera,
        scale: BillboardScale::SignedFactors,
    };
    source
        .edit(|builder| builder.set_node(mount, node))
        .unwrap();
    // spawn_pair evaluates during Update; propagation occurs later in PostUpdate.
    let (mut app, root, _, _) = spawn_pair(&source);
    let driver = app.world_mut().spawn(Transform::default()).id();
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .driving_camera = Some(driver);
    app.update();
    let rig = app
        .world()
        .get::<ModelRig>(root)
        .unwrap()
        .node(mount)
        .unwrap();
    let initial = *app.world().get::<Transform>(rig).unwrap();
    let count = app
        .world()
        .get::<ModelRig>(root)
        .unwrap()
        .evaluation_count();
    app.update();
    assert_eq!(
        app.world()
            .get::<ModelRig>(root)
            .unwrap()
            .evaluation_count(),
        count
    );
    // No propagation occurs between these local edits and Update evaluation.
    app.world_mut()
        .get_mut::<Transform>(driver)
        .unwrap()
        .rotation = Quat::from_rotation_y(0.7);
    app.world_mut().get_mut::<Transform>(root).unwrap().rotation = Quat::from_rotation_y(0.2);
    app.update();
    let next = *app.world().get::<Transform>(rig).unwrap();
    assert_ne!(initial.rotation, next.rotation);
    let basis = app
        .world()
        .get::<ModelRig>(root)
        .unwrap()
        .model()
        .model_to_bevy()
        .to_matrix();
    let root_matrix = app.world().get::<Transform>(root).unwrap().to_matrix();
    let camera_matrix = app.world().get::<Transform>(driver).unwrap().to_matrix();
    let relative = (root_matrix * basis).inverse() * camera_matrix;
    let expected = Quat::from_mat3(&Mat3::from_mat4(relative)).normalize();
    assert!(next.rotation.abs_diff_eq(expected, 1e-5));
    let count = app
        .world()
        .get::<ModelRig>(root)
        .unwrap()
        .evaluation_count();
    app.update();
    assert_eq!(
        app.world()
            .get::<ModelRig>(root)
            .unwrap()
            .evaluation_count(),
        count
    );
}
