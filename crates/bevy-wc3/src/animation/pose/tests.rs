use super::*;
use bevy::ecs::system::SystemState;
use bevy::transform::TransformSystems;
use std::f32::consts::FRAC_PI_2;
use wc3::model::animation::{Track, ValueKeyframe};
use wc3::model::scene::NodeFlags;

use crate::animation::pose::sample_emitter_transform;
use crate::assets::model::Wc3Model;

fn animation() -> Wc3Animation {
    let model =
        Wc3Model::decode_mdl(include_str!("../../../tests/fixtures/geoset_capture.mdl")).unwrap();
    Wc3Animation {
        sequence: 0,
        elapsed_ms: 0.0,
        speed: 1.0,
        playing: false,
        sequences: model.model.sequences(),
        global_sequences: vec![],
    }
}

fn vector_track(from: Vec3, to: Vec3) -> Track<[f32; 3]> {
    Track::linear(
        vec![
            ValueKeyframe {
                frame: 0,
                value: from.to_array(),
            },
            ValueKeyframe {
                frame: 1000,
                value: to.to_array(),
            },
        ],
        None,
    )
    .unwrap()
}

fn node(root: Entity, pivot: Vec3, parent_pivot: Vec3, bits: u32) -> AnimatedNode {
    AnimatedNode {
        root,
        pivot,
        parent_pivot,
        flags: NodeFlags(bits),
        camera: None,
        translation: None,
        rotation: None,
        scaling: None,
    }
}

fn scene() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin));
    app.add_systems(
        PostUpdate,
        animate_nodes.before(TransformSystems::Propagate),
    );
    let root = app
        .world_mut()
        .spawn((Transform::default(), animation()))
        .id();
    (app, root)
}

fn spawn_node(app: &mut App, parent: Entity, node: AnimatedNode) -> Entity {
    app.world_mut()
        .spawn((node, Transform::default(), ChildOf(parent)))
        .id()
}

fn close(actual: Vec3, expected: Vec3) {
    assert!(
        actual.abs_diff_eq(expected, 1e-4),
        "{actual:?} != {expected:?}"
    );
}

fn global(app: &App, entity: Entity) -> GlobalTransform {
    *app.world().get::<GlobalTransform>(entity).unwrap()
}

#[test]
fn inheritance_suppresses_animation_but_preserves_model_placement_and_pivots() {
    let (mut app, root) = scene();
    let placement = Transform {
        translation: Vec3::new(10.0, 20.0, 30.0),
        rotation: Quat::from_rotation_z(0.3),
        scale: Vec3::splat(2.0),
    };
    *app.world_mut().get_mut::<Transform>(root).unwrap() = placement;
    let mut parent_node = node(root, Vec3::X, Vec3::ZERO, 0);
    parent_node.translation = Some(vector_track(Vec3::Y * 3.0, Vec3::Y * 3.0));
    parent_node.scaling = Some(vector_track(Vec3::splat(3.0), Vec3::splat(3.0)));
    parent_node.rotation = Some(
        Track::linear(
            vec![ValueKeyframe {
                frame: 0,
                value: Quat::from_rotation_z(0.7).to_array(),
            }],
            None,
        )
        .unwrap(),
    );
    let parent = spawn_node(&mut app, root, parent_node);
    let ordinary = spawn_node(&mut app, parent, node(root, Vec3::X * 2.0, Vec3::X, 0));
    let flagged = spawn_node(&mut app, parent, node(root, Vec3::X * 2.0, Vec3::X, 7));
    let child = app
        .world_mut()
        .spawn((Transform::from_translation(Vec3::Y), ChildOf(flagged)))
        .id();
    app.update();
    let expected_position =
        placement.transform_point(Vec3::X + Quat::from_rotation_z(0.7) * Vec3::X * 3.0);
    close(global(&app, flagged).translation(), expected_position);
    close(
        global(&app, ordinary).translation(),
        expected_position + placement.rotation * (Vec3::Y * 6.0),
    );
    close(
        global(&app, flagged).affine().transform_vector3(Vec3::X),
        placement.rotation * (Vec3::X * 2.0),
    );
    close(
        global(&app, child).translation(),
        expected_position + placement.rotation * (Vec3::Y * 2.0),
    );
}

#[test]
fn translation_suppression_is_deterministic_when_seeking_and_paused() {
    let (mut app, root) = scene();
    let mut parent_node = node(root, Vec3::ZERO, Vec3::ZERO, 0);
    parent_node.translation = Some(vector_track(Vec3::ZERO, Vec3::X * 8.0));
    let parent = spawn_node(&mut app, root, parent_node);
    let mut child = node(root, Vec3::Y, Vec3::ZERO, 1);
    child.translation = Some(vector_track(Vec3::ZERO, Vec3::Z * 4.0));
    let child = spawn_node(&mut app, parent, child);
    for time in [1000.0, 250.0, 250.0, 0.0] {
        app.world_mut()
            .get_mut::<Wc3Animation>(root)
            .unwrap()
            .elapsed_ms = time;
        app.update();
        close(
            global(&app, child).translation(),
            Vec3::Y + Vec3::Z * (time as f32 / 250.0),
        );
    }
}

#[test]
fn full_billboard_uses_current_camera_and_retains_authored_rotation() {
    let (mut app, root) = scene();
    let mut bone = node(root, Vec3::ZERO, Vec3::ZERO, 8);
    let authored = Quat::from_rotation_x(0.2);
    bone.rotation = Some(
        Track::linear(
            vec![ValueKeyframe {
                frame: 0,
                value: authored.to_array(),
            }],
            None,
        )
        .unwrap(),
    );
    let bone = spawn_node(&mut app, root, bone);
    let camera = app
        .world_mut()
        .spawn((Camera3d::default(), Transform::default()))
        .id();
    for rotation in [
        Quat::IDENTITY,
        Quat::from_rotation_y(0.6),
        Quat::from_rotation_z(1.1),
    ] {
        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .rotation = rotation;
        app.update();
        let expected = rotation
            * Quat::from_rotation_y(-FRAC_PI_2)
            * Quat::from_rotation_x(-FRAC_PI_2)
            * authored;
        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
            close(
                global(&app, bone).affine().transform_vector3(axis),
                expected * axis,
            );
        }
    }
}

#[test]
fn billboard_axis_locks_keep_the_selected_axis_and_face_camera() {
    for (bits, axis, forward) in [
        (16, Vec3::X, Vec3::Y),
        (32, Vec3::Y, Vec3::X),
        (64, Vec3::Z, Vec3::X),
    ] {
        let (mut app, root) = scene();
        let camera_rotation = Quat::from_rotation_y(0.7) * Quat::from_rotation_x(0.4);
        app.world_mut().spawn((
            Camera3d::default(),
            Transform::from_rotation(camera_rotation),
        ));
        let bone = spawn_node(&mut app, root, node(root, Vec3::ZERO, Vec3::ZERO, bits));
        app.update();
        let pose = global(&app, bone);
        close(pose.affine().transform_vector3(axis), axis);
        let ray = camera_rotation * Vec3::Z;
        let projected = (ray - axis * ray.dot(axis)).normalize();
        close(pose.affine().transform_vector3(forward), projected);
    }
}

#[test]
fn camera_anchoring_applies_world_camera_translation_once_in_a_chain() {
    let (mut app, root) = scene();
    *app.world_mut().get_mut::<Transform>(root).unwrap() = Transform::from_xyz(3.0, 0.0, 0.0)
        .with_rotation(Quat::from_rotation_z(0.7))
        .with_scale(Vec3::splat(2.0));
    let camera = app
        .world_mut()
        .spawn((Camera3d::default(), Transform::from_xyz(10.0, 20.0, 30.0)))
        .id();
    let first = spawn_node(&mut app, root, node(root, Vec3::X, Vec3::ZERO, 128));
    let second = spawn_node(&mut app, first, node(root, Vec3::X * 2.0, Vec3::X, 128));
    app.update();
    let placement = *app.world().get::<Transform>(root).unwrap();
    close(
        global(&app, second).translation(),
        placement.transform_point(Vec3::X * 2.0) + Vec3::new(10.0, 20.0, 30.0),
    );
    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation += Vec3::Y;
    app.update();
    close(
        global(&app, second).translation(),
        placement.transform_point(Vec3::X * 2.0) + Vec3::new(10.0, 21.0, 30.0),
    );
}

#[test]
fn camera_selection_prefers_window_then_marker_then_instance_override() {
    let (mut app, root) = scene();
    let bone = spawn_node(&mut app, root, node(root, Vec3::ZERO, Vec3::ZERO, 128));
    let window = app
        .world_mut()
        .spawn((Camera3d::default(), Transform::from_translation(Vec3::X)))
        .id();
    let preview = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            RenderTarget::Image(Handle::<Image>::default().into()),
            Transform::from_translation(Vec3::Y),
        ))
        .id();
    app.world_mut()
        .spawn((Camera2d, Transform::from_translation(Vec3::Z)));
    app.update();
    close(global(&app, bone).translation(), Vec3::X);
    app.world_mut()
        .entity_mut(preview)
        .insert(Wc3DefaultNodeCamera);
    app.update();
    close(global(&app, bone).translation(), Vec3::Y);
    app.world_mut()
        .entity_mut(root)
        .insert(Wc3NodeCamera(window));
    app.update();
    close(global(&app, bone).translation(), Vec3::X);
    app.world_mut().get_mut::<Camera>(window).unwrap().is_active = false;
    app.update();
    close(global(&app, bone).translation(), Vec3::ZERO);
    app.world_mut().entity_mut(root).remove::<Wc3NodeCamera>();
    app.update();
    close(global(&app, bone).translation(), Vec3::Y);
}

#[test]
fn ambiguous_cameras_skip_only_camera_dependent_flags_and_recover() {
    let (mut app, root) = scene();
    let mut bone = node(root, Vec3::X, Vec3::ZERO, 128);
    bone.translation = Some(vector_track(Vec3::Y, Vec3::Y));
    let bone = spawn_node(&mut app, root, bone);
    app.world_mut()
        .spawn((Camera3d::default(), Transform::from_translation(Vec3::Z)));
    let other = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            Transform::from_translation(Vec3::Z * 2.0),
        ))
        .id();
    app.update();
    close(global(&app, bone).translation(), Vec3::X + Vec3::Y);
    app.world_mut()
        .entity_mut(other)
        .insert(Wc3DefaultNodeCamera);
    app.update();
    close(
        global(&app, bone).translation(),
        Vec3::X + Vec3::Y + Vec3::Z * 2.0,
    );
    app.world_mut().despawn(other);
    app.update();
    close(
        global(&app, bone).translation(),
        Vec3::X + Vec3::Y + Vec3::Z,
    );
}

#[test]
fn attached_instances_inherit_camera_selection_and_can_override_it() {
    let (mut app, root) = scene();
    let first = app
        .world_mut()
        .spawn((Camera3d::default(), Transform::from_translation(Vec3::X)))
        .id();
    let second = app
        .world_mut()
        .spawn((Camera3d::default(), Transform::from_translation(Vec3::Y)))
        .id();
    app.world_mut()
        .entity_mut(root)
        .insert(Wc3NodeCamera(first));
    let mount = spawn_node(&mut app, root, node(root, Vec3::ZERO, Vec3::ZERO, 0));
    let attached = app
        .world_mut()
        .spawn((Transform::default(), animation(), ChildOf(mount)))
        .id();
    let child = spawn_node(
        &mut app,
        attached,
        node(attached, Vec3::ZERO, Vec3::ZERO, 128),
    );
    app.update();
    close(global(&app, child).translation(), Vec3::X);
    app.world_mut()
        .entity_mut(attached)
        .insert(Wc3NodeCamera(second));
    app.update();
    close(global(&app, child).translation(), Vec3::Y);
}

#[test]
fn birth_sampling_evaluates_flags_at_birth_instead_of_using_endpoint_pose() {
    let (mut app, root) = scene();
    let mut parent_node = node(root, Vec3::ZERO, Vec3::ZERO, 0);
    parent_node.translation = Some(vector_track(Vec3::ZERO, Vec3::X * 8.0));
    let parent = spawn_node(&mut app, root, parent_node);
    let mut bone = node(root, Vec3::Y, Vec3::ZERO, 1 | 128);
    bone.translation = Some(vector_track(Vec3::ZERO, Vec3::Z * 4.0));
    let bone = spawn_node(&mut app, parent, bone);
    app.world_mut().spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::X * 10.0),
    ));
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .elapsed_ms = 1000.0;
    app.update();
    let mut at_birth = animation();
    at_birth.elapsed_ms = 250.0;
    let mut state = SystemState::<
        Query<(
            &GlobalTransform,
            Option<&Transform>,
            Option<&AnimatedNode>,
            Option<&ChildOf>,
        )>,
    >::new(app.world_mut());
    let query = state.get(app.world()).unwrap();
    let sampled = sample_emitter_transform(bone, root, &at_birth, &query).unwrap();
    close(sampled.translation(), Vec3::X * 10.0 + Vec3::Y + Vec3::Z);
    close(
        global(&app, bone).translation(),
        Vec3::X * 10.0 + Vec3::Y + Vec3::Z * 4.0,
    );
}

#[test]
fn mirrored_and_collapsed_parent_scales_remain_finite() {
    for scale in [Vec3::new(-2.0, 3.0, 4.0), Vec3::new(0.0, 3.0, 4.0)] {
        let (mut app, root) = scene();
        let mut parent_node = node(root, Vec3::ZERO, Vec3::ZERO, 0);
        parent_node.scaling = Some(vector_track(scale, scale));
        let parent = spawn_node(&mut app, root, parent_node);
        let child = spawn_node(&mut app, parent, node(root, Vec3::ZERO, Vec3::ZERO, 4));
        app.update();
        let affine = global(&app, child).affine();
        assert!(affine.is_finite());
        close(affine.transform_vector3(Vec3::Y), Vec3::Y);
        close(affine.transform_vector3(Vec3::Z), Vec3::Z);
        close(
            affine.transform_vector3(Vec3::X),
            if scale.x == 0.0 { Vec3::ZERO } else { Vec3::X },
        );
    }
}

#[test]
fn hierarchy_cycles_fail_sampling_without_recursion_overflow() {
    let a = Entity::from_bits(1);
    let b = Entity::from_bits(2);
    let lookup = |entity| {
        Some(PoseInput {
            local: Transform::default(),
            parent: Some(if entity == a { b } else { a }),
            node: None,
            world: None,
            anchor: None,
        })
    };
    assert!(resolve_pose(a, &lookup, &mut HashMap::new(), &mut HashSet::new()).is_none());
}

fn sampled_global(app: &mut App, entity: Entity, root: Entity) -> GlobalTransform {
    let animation = app.world().get::<Wc3Animation>(root).unwrap().clone();
    let mut state = SystemState::<
        Query<(
            &GlobalTransform,
            Option<&Transform>,
            Option<&AnimatedNode>,
            Option<&ChildOf>,
        )>,
    >::new(app.world_mut());
    let query = state.get(app.world()).unwrap();
    sample_emitter_transform(entity, root, &animation, &query).unwrap()
}

#[test]
fn birth_sampling_preserves_signed_instance_scale_and_rotation() {
    let (mut app, root) = scene();
    let placement = Transform::from_xyz(2.0, 3.0, 4.0)
        .with_rotation(Quat::from_rotation_z(0.3))
        .with_scale(Vec3::new(2.0, -3.0, 4.0));
    *app.world_mut().get_mut::<Transform>(root).unwrap() = placement;
    let mut parent_node = node(root, Vec3::ZERO, Vec3::ZERO, 0);
    parent_node.rotation = Some(
        Track::linear(
            vec![ValueKeyframe {
                frame: 0,
                value: Quat::from_rotation_z(0.7).to_array(),
            }],
            None,
        )
        .unwrap(),
    );
    parent_node.scaling = Some(vector_track(Vec3::splat(2.0), Vec3::splat(2.0)));
    let parent = spawn_node(&mut app, root, parent_node);
    let child = spawn_node(&mut app, parent, node(root, Vec3::X, Vec3::ZERO, 7));
    app.update();
    let sampled = sampled_global(&mut app, child, root);
    let current = global(&app, child);
    close(sampled.translation(), current.translation());
    for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
        close(
            sampled.affine().transform_vector3(axis),
            current.affine().transform_vector3(axis),
        );
    }
}

#[test]
fn attached_anchored_nodes_sample_the_existing_camera_offset_once() {
    let (mut app, root) = scene();
    app.world_mut().spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::X * 10.0),
    ));
    let mount = spawn_node(&mut app, root, node(root, Vec3::Y, Vec3::ZERO, 128));
    let attached = app
        .world_mut()
        .spawn((
            Transform::from_translation(Vec3::Z),
            animation(),
            ChildOf(mount),
        ))
        .id();
    let child = spawn_node(
        &mut app,
        attached,
        node(attached, Vec3::ZERO, Vec3::ZERO, 128),
    );
    app.update();
    let sampled = sampled_global(&mut app, child, attached);
    close(sampled.translation(), Vec3::X * 10.0 + Vec3::Y + Vec3::Z);
    close(sampled.translation(), global(&app, child).translation());
}

#[test]
fn rotation_suppression_and_axis_lock_compose_in_the_effective_parent_frame() {
    let (mut app, root) = scene();
    let mut parent_node = node(root, Vec3::ZERO, Vec3::ZERO, 0);
    parent_node.rotation = Some(
        Track::linear(
            vec![ValueKeyframe {
                frame: 0,
                value: Quat::from_rotation_z(0.7).to_array(),
            }],
            None,
        )
        .unwrap(),
    );
    let parent = spawn_node(&mut app, root, parent_node);
    let camera_rotation = Quat::from_rotation_y(0.7) * Quat::from_rotation_x(0.4);
    app.world_mut().spawn((
        Camera3d::default(),
        Transform::from_rotation(camera_rotation),
    ));
    let child = spawn_node(&mut app, parent, node(root, Vec3::ZERO, Vec3::ZERO, 2 | 64));
    app.update();
    close(
        global(&app, child).affine().transform_vector3(Vec3::Z),
        Vec3::Z,
    );
    let ray = camera_rotation * Vec3::Z;
    close(
        global(&app, child).affine().transform_vector3(Vec3::X),
        Vec3::new(ray.x, ray.y, 0.0).normalize(),
    );
}

#[test]
fn node_flags_survive_mdl_loading_and_reach_the_spawned_rig() {
    use crate::instance::spawn::{spawn_prepared_model, Wc3NodeEntities};
    use crate::materials::Wc3LayerMaterial;
    use crate::preparation::prepare_model;
    use bevy::ecs::world::CommandQueue;
    use bevy::mesh::skinning::SkinnedMeshInverseBindposes;

    let source = include_str!("../../../tests/fixtures/attachment_capture_child.mdl")
        .replace("ObjectId 0,", "ObjectId 0, DontInheritTranslation, DontInheritRotation, DontInheritScaling, Billboarded, BillboardedLockX, BillboardedLockY, BillboardedLockZ, CameraAnchored,");
    let model = Wc3Model::decode_mdl(&source).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut bindposes, &model, |_| None).unwrap();
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    let root = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    queue.apply(&mut world);
    let bone = world.get::<Wc3NodeEntities>(root).unwrap().get(0).unwrap();
    assert_eq!(
        world.get::<AnimatedNode>(bone).unwrap().flags.bits() & 0xff,
        0xff
    );
}
