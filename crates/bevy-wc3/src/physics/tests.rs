use super::*;
use bevy::math::Mat4;
use bevy::math::Vec3;
use std::f64::consts::FRAC_PI_2;
use wc3::model::systems::model_physics::Spring;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{ModelMount, MountTarget};
use wc3::model::{Node, Quat};
fn fixture(dynamic: usize) -> (PhysicsDefinition, Vec<Mat4>) {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let mut bodies = Vec::new();
    let mut authored = Vec::new();
    for index in 0..dynamic {
        let id = builder
            .add_node(Node::named(format!("body-{index}"), &mut slot_values))
            .unwrap();
        bodies.push(BodyDefinition {
            info: PhysicsBodyInfo {
                domain: PhysicsKind::WornItem,
                target: id,
                body: Body {
                    node: id,
                    kind: PhysicsBodyKind::Dynamic,
                    gravity_scale: 1.,
                    inertia_scale: 1.,
                    linear_damping: 0.,
                    angular_damping: 0.,
                    follow_factor: 0.,
                    parent_body: None,
                    attachment_host: false,
                    ragdoll_root: false,
                    shapes: Vec::new(),
                },
            },
            placement: index,
            target: index,
            offset_inverse: Mat4::IDENTITY,
            gravity: 1.,
            damping: 0.,
            angular_damping: 0.,
            follow: 0.,
            inverse_mass: 1.,
            inverse_inertia: Mat3::IDENTITY,
            mass: Mass {
                mass: 1.,
                center: Vec3::ZERO,
                second_moment: Mat3::from_diagonal(Vec3::splat(0.5)),
            },
            fixtures: vec![Fixture::test_primitive(
                contact::Primitive::Sphere {
                    center: Vec3::ZERO,
                    radius: 0.5,
                },
                Vec3::ZERO,
                0.5,
                0.,
            )],
            radius: 0.5,
        });
        authored.push(Mat4::from_translation(Vec3::X * index as f32));
    }
    (
        PhysicsDefinition {
            bodies,
            ..SlotConstruct::construct(&mut slot_values)
        },
        authored,
    )
}
fn translation(output: &[Option<Mat4>], index: usize) -> Vec3 {
    output[index].unwrap().w_axis.truncate()
}

#[test]
fn fixed_steps_match_across_frame_rates() {
    let (definition, authored) = fixture(1);
    let settings = PhysicsSimulation::default();
    let mut slow = PhysicsState::default();
    let mut fast = PhysicsState::default();
    for frame in 0..=60 {
        slow.step(
            &definition,
            &settings,
            frame as f64 / 60.,
            &authored,
            &HashMap::new(),
        );
    }
    for frame in 0..=120 {
        fast.step(
            &definition,
            &settings,
            frame as f64 / 120.,
            &authored,
            &HashMap::new(),
        );
    }
    assert!((translation(&slow.output, 0) - translation(&fast.output, 0)).length() < 1e-5);
    assert!((translation(&fast.output, 0).z + 4.945875).abs() < 1e-4);
}

#[test]
fn seek_and_explicit_reset_clear_velocity_without_catchup() {
    let (definition, authored) = fixture(1);
    let mut settings = PhysicsSimulation::default();
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.step(&definition, &settings, 0.1, &authored, &HashMap::new());
    assert!(translation(&state.output, 0).z < 0.);
    state.step(&definition, &settings, 5., &authored, &HashMap::new());
    assert_eq!(translation(&state.output, 0), Vec3::ZERO);
    state.step(&definition, &settings, 5.1, &authored, &HashMap::new());
    assert!(translation(&state.output, 0).z < 0.);
    settings.reset();
    state.step(&definition, &settings, 5.1, &authored, &HashMap::new());
    assert_eq!(translation(&state.output, 0), Vec3::ZERO);
}

#[test]
fn placement_offset_drives_bone_instead_of_placement_only() {
    let (mut definition, mut authored) = fixture(1);
    let offset = Vec3::new(0., 0., 2.);
    definition.bodies[0].offset_inverse = Mat4::from_translation(-offset);
    authored[0] = Mat4::from_translation(offset);
    let settings = PhysicsSimulation::default();
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    assert_eq!(translation(&state.output, 0), Vec3::ZERO);
    state.step(&definition, &settings, 0.1, &authored, &HashMap::new());
    let driven = translation(&state.output, 0);
    assert!(driven.z < 0. && driven.z > -0.1);
}

#[test]
fn distance_joint_preserves_anchor_separation_and_override_authority() {
    let (mut definition, authored) = fixture(2);
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    definition.joints.push(JointDefinition {
        a: 0,
        b: 1,
        anchor_a: Vec3::ZERO,
        anchor_b: Vec3::ZERO,
        rotation_a: Quat::IDENTITY,
        rotation_b: Quat::IDENTITY,
        authored: Joint {
            body_a: definition.bodies[0].info.body.node,
            body_b: definition.bodies[1].info.body.node,
            frame_a: Frame {
                axis_x: Vec3::X,
                axis_y: Vec3::Y,
                axis_z: Vec3::Z,
                origin: Vec3::from_array([0.; 3]),
            },
            frame_b: Frame {
                axis_x: Vec3::X,
                axis_y: Vec3::Y,
                axis_z: Vec3::Z,
                origin: Vec3::from_array([0.; 3]),
            },
            constraint: Constraint::Distance {
                distance: 1.,
                spring: Spring::default(),
            },
        },
    });
    let settings = PhysicsSimulation::default();
    let mut state = PhysicsState::default();
    for frame in 0..=120 {
        state.step(
            &definition,
            &settings,
            frame as f64 / 120.,
            &authored,
            &HashMap::new(),
        );
    }
    assert!((translation(&state.output, 1).length() - 1.).abs() < 1e-5);
    assert!(translation(&state.output, 1).z < -0.1);
    let overrides = HashMap::from([(definition.bodies[1].info.target, ModelTransform::default())]);
    state.step(&definition, &settings, 1.1, &authored, &overrides);
    assert!(state.output[1].is_none());
    assert_eq!(state.bodies[1].position, Vec3::X);
}

#[test]
fn collisions_require_explicit_pair_and_respect_kinematic_body() {
    let mut slot_values = SlotValues::default();
    let (mut definition, mut authored) = fixture(2);
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    authored[1] = Mat4::from_translation(Vec3::X * 0.25);
    let settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.step(&definition, &settings, 0.1, &authored, &HashMap::new());
    assert_eq!(translation(&state.output, 1), Vec3::X * 0.25);
    definition.pairs.push((0, 1));
    state.step(&definition, &settings, 0.2, &authored, &HashMap::new());
    assert!(translation(&state.output, 1).x >= 1.);
    assert_eq!(state.bodies[0].position, Vec3::ZERO);
}

#[test]
fn paused_preserves_state_and_invalid_settings_do_not_emit_nonfinite_poses() {
    let (definition, authored) = fixture(1);
    let mut settings = PhysicsSimulation::default();
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.step(&definition, &settings, 0.1, &authored, &HashMap::new());
    let before = translation(&state.output, 0);
    settings.paused = true;
    state.step(&definition, &settings, 0.2, &authored, &HashMap::new());
    assert_eq!(translation(&state.output, 0), before);
    settings.gravity.x = f32::NAN;
    assert!(state
        .step(&definition, &settings, 0.3, &authored, &HashMap::new())
        .is_empty());
}

#[test]
fn runtime_simulates_rest_rig_and_restores_authored_pose_when_disabled() {
    let mut slot_values = SlotValues::default();
    use crate::runtime::animate_instances;
    use crate::{compile_model, spawn_model, ModelMaterial, ModelRig, Playback, PoseOverrides};
    use bevy::ecs::world::CommandQueue;
    use wc3::model::systems::model_physics::{Body, Physics};
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let bone = builder.add_named_node("armor-bone").unwrap();
    let placement = builder
        .add_node(
            Node::named("physics-body", &mut slot_values)
                .with_parent(bone)
                .with_transform(
                    ModelTransform {
                        translation: Vec3::from_array([0., 0., 1.]),
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    &mut slot_values,
                )
                .unwrap(),
        )
        .unwrap();
    builder
        .add_system(System::M2(ModelSystem::Physics(Physics {
            kind: PhysicsKind::WornItem,
            resource: None,
            bodies: vec![Body {
                node: placement,
                kind: PhysicsBodyKind::Dynamic,
                gravity_scale: 1.,
                inertia_scale: 1.,
                linear_damping: 0.,
                angular_damping: 0.,
                follow_factor: 0.,
                parent_body: None,
                attachment_host: false,
                ragdoll_root: false,
                shapes: Vec::new(),
            }],
            joints: Vec::new(),
            collision_pairs: Vec::new(),
            wind_coefficients: None,
            follow_allow_list: SlotConstruct::construct(&mut slot_values),
            vegetation: Vec::new(),
        })))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut materials = Assets::<ModelMaterial>::default();
    let mut app = App::new();
    app.add_systems(Update, animate_instances);
    let mut queue = CommandQueue::default();
    let root = spawn_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    queue.apply(app.world_mut());
    app.insert_resource(materials);
    app.world_mut()
        .entity_mut(root)
        .insert(PhysicsSimulation::default());
    app.update();
    let entity = app
        .world()
        .get::<ModelRig>(root)
        .unwrap()
        .node(bone)
        .unwrap();
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .shared_time = 0.1;
    app.update();
    assert!(app.world().get::<Transform>(entity).unwrap().translation.z < 0.);
    app.world_mut()
        .get_mut::<PoseOverrides>(root)
        .unwrap()
        .0
        .insert(
            bone,
            ModelTransform {
                translation: Vec3::from_array([0., 0., 3.]),
                ..SlotConstruct::construct(&mut slot_values)
            },
        );
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .shared_time = 0.2;
    app.update();
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap().translation.z,
        3.
    );
    app.world_mut()
        .get_mut::<PoseOverrides>(root)
        .unwrap()
        .0
        .clear();
    app.world_mut()
        .entity_mut(root)
        .remove::<PhysicsSimulation>();
    app.update();
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap().translation,
        Vec3::ZERO
    );
}

#[test]
fn composition_remaps_parent_bodies_and_constraints_to_shared_host_body() {
    let mut slot_values = SlotValues::default();
    let (mut host, _) = fixture(1);
    let (mut part, _) = fixture(2);
    let host_id = host.bodies[0].info.body.node;
    part.bodies[1].info.body.parent_body = Some(part.bodies[0].info.body.node);
    let frame = Frame {
        axis_x: Vec3::X,
        axis_y: Vec3::Y,
        axis_z: Vec3::Z,
        origin: Vec3::from_array([0.; 3]),
    };
    let authored = Joint {
        body_a: part.bodies[0].info.body.node,
        body_b: part.bodies[1].info.body.node,
        frame_a: frame,
        frame_b: frame,
        constraint: Constraint::Distance {
            distance: 1.,
            spring: Spring::default(),
        },
    };
    part.joints.push(JointDefinition {
        a: 0,
        b: 1,
        anchor_a: Vec3::ZERO,
        anchor_b: Vec3::ZERO,
        rotation_a: Quat::IDENTITY,
        rotation_b: Quat::IDENTITY,
        authored,
    });
    part.pairs.push((0, 1));
    let ids = [host_id, part.bodies[1].info.body.node];
    let nodes: Vec<_> = ids
        .into_iter()
        .map(|id| RuntimeNode {
            id,
            name: "composed".into(),
            parent: None,
            transform: wc3::model::AnimatedTransform::construct(&mut slot_values),
            visibility: slot_values.add(true),
            rest: ModelTransform::default(),
            channels: SlotConstruct::construct(&mut slot_values),
            inheritance: SlotConstruct::construct(&mut slot_values),
            billboard: SlotConstruct::construct(&mut slot_values),
            transform_semantics: SlotConstruct::construct(&mut slot_values),
            visible: true,
            camera_anchored: false,
            correction: None,
        })
        .collect();
    host.append_remapped(&part, &[0, 1], &nodes);
    assert_eq!(host.bodies.len(), 2);
    assert_eq!(host.bodies[1].info.body.parent_body, Some(host_id));
    assert_eq!(host.joints.len(), 1);
    assert_eq!(host.joints[0].authored.body_a, host_id);
    assert_eq!(host.joints[0].authored.body_b, ids[1]);
    assert_eq!(host.pairs, [(0, 1)]);
}

#[test]
fn composition_body_offset_includes_rest_correction_anchors() {
    let mut slot_values = SlotValues::default();
    let (mut part, _) = fixture(1);
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let host = builder.add_named_node("host").unwrap();
    let anchor = builder.add_named_node("rest correction").unwrap();
    let placement = builder.add_named_node("placement").unwrap();
    let mut make = |id, parent, x| RuntimeNode {
        transform: wc3::model::AnimatedTransform::construct(&mut slot_values),
        visibility: slot_values.add(true),
        id,
        name: "corrected".into(),
        channels: SlotConstruct::construct(&mut slot_values),
        parent,
        rest: ModelTransform {
            translation: Vec3::from_array([x, 0., 0.]),
            ..SlotConstruct::construct(&mut slot_values)
        },
        inheritance: SlotConstruct::construct(&mut slot_values),
        billboard: SlotConstruct::construct(&mut slot_values),
        transform_semantics: SlotConstruct::construct(&mut slot_values),
        visible: true,
        camera_anchored: false,
        correction: None,
    };
    let nodes = [
        make(host, None, 4.),
        make(anchor, Some(0), 3.),
        make(placement, Some(1), 2.),
    ];
    // Source placement=0 drives source target=1; destination target is host0,
    // with placement2 beneath a rest correction anchor1.
    part.bodies[0].placement = 0;
    part.bodies[0].target = 1;
    let mut composed = PhysicsDefinition::default();
    composed.append_remapped(&part, &[2, 0], &nodes);
    let authored = [
        Mat4::from_translation(Vec3::X * 4.),
        Mat4::from_translation(Vec3::X * 7.),
        Mat4::from_translation(Vec3::X * 9.),
    ];
    let mut state = PhysicsState::default();
    let output = state.step(
        &composed,
        &PhysicsSimulation::default(),
        0.,
        &authored,
        &HashMap::new(),
    );
    assert_eq!(translation(output, 0), Vec3::X * 4.);
}

#[test]
fn shoulder_limits_separate_swing_and_twist_about_frame_z() {
    let original = Quat::from_rotation_x(1.2) * Quat::from_rotation_z(0.8);
    let limited = limited_rotation(original, [-0.2, 0.2], 0.4);
    let axis = limited * Vec3::Z;
    assert!((axis.angle_between(Vec3::Z) - 0.4).abs() < 1e-5);
    let twist = Quat::from_xyzw(0., 0., limited.z, limited.w).normalize();
    assert!((2. * twist.z.atan2(twist.w) - 0.2).abs() < 1e-5);
    let hinge = limited_rotation(original, [-0.2, 0.2], 0.);
    assert!((hinge * Vec3::Z - Vec3::Z).length() < 1e-5);
    assert!(limited_rotation(Quat::from_xyzw(1., 0., 0., 0.), [-0.2, 0.2], 0.4).is_finite());
}

#[test]
fn offset_mass_center_creates_gravity_torque_about_pinned_body_origin() {
    let (mut definition, mut authored) = fixture(2);
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    definition.bodies[1].mass.center = Vec3::new(1., 0., -1.);
    authored[1] = Mat4::IDENTITY;
    let frame = Frame {
        axis_x: Vec3::X,
        axis_y: Vec3::Y,
        axis_z: Vec3::Z,
        origin: Vec3::from_array([0.; 3]),
    };
    definition.joints.push(JointDefinition {
        a: 0,
        b: 1,
        anchor_a: Vec3::ZERO,
        anchor_b: Vec3::ZERO,
        rotation_a: Quat::IDENTITY,
        rotation_b: Quat::IDENTITY,
        authored: Joint {
            body_a: definition.bodies[0].info.body.node,
            body_b: definition.bodies[1].info.body.node,
            frame_a: frame,
            frame_b: frame,
            constraint: Constraint::Spherical {
                friction_torque: 0.,
            },
        },
    });
    let mut state = PhysicsState::default();
    for frame in 0..=60 {
        state.step(
            &definition,
            &PhysicsSimulation::default(),
            frame as f64 / 120.,
            &authored,
            &HashMap::new(),
        );
    }
    let matrix = state.output[1].unwrap();
    let (_, rotation, origin) = matrix.to_scale_rotation_translation();
    assert!(matrix.is_finite());
    assert!(rotation.angle_between(Quat::IDENTITY) > 0.1);
    assert!(
        origin.length() < 0.01,
        "body joint origin drifted: {origin:?}"
    );
}

#[test]
fn explicit_contact_supports_dynamic_body_under_gravity_without_energy_growth() {
    let (mut definition, mut authored) = fixture(2);
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    definition.pairs.push((0, 1));
    authored[1] = Mat4::from_translation(Vec3::Z);
    let mut state = PhysicsState::default();
    for frame in 0..=600 {
        let output = state.step(
            &definition,
            &PhysicsSimulation::default(),
            frame as f64 / 120.,
            &authored,
            &HashMap::new(),
        );
        assert!(output[1].unwrap().is_finite());
        assert!((translation(output, 1) - Vec3::Z).length() < 1e-4);
    }
    assert!(state.bodies[1].velocity.length() < 1e-4);
}

#[test]
fn mounted_physics_inherits_forces_through_current_frame_carrier_rotation() {
    let mut slot_values = SlotValues::default();
    use crate::runtime::{animate_instances, synchronize_mounted_models};
    use crate::{
        compile_model, spawn_model, ModelMaterial, ModelPart, ModelRig, PartBinding, Playback,
        PoseOverrides,
    };
    use bevy::ecs::world::CommandQueue;
    use wc3::model::systems::model_physics::Physics;
    let mut host = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let carrier = host.add_named_node("carrier").unwrap();
    let host = compile_model(
        &{
            host.import_values(&slot_values).unwrap();
            host.finish()
        }
        .unwrap(),
        |_| None,
    )
    .unwrap();
    let mut part = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let bone = part.add_named_node("bone").unwrap();
    let placement = part
        .add_node(Node::named("body", &mut slot_values).with_parent(bone))
        .unwrap();
    part.add_system(System::M2(ModelSystem::Physics(Physics {
        kind: PhysicsKind::WornItem,
        resource: None,
        bodies: vec![Body {
            node: placement,
            kind: PhysicsBodyKind::Dynamic,
            gravity_scale: 1.,
            inertia_scale: 1.,
            linear_damping: 0.,
            angular_damping: 0.,
            follow_factor: 0.,
            parent_body: None,
            attachment_host: false,
            ragdoll_root: false,
            shapes: Vec::new(),
        }],
        joints: Vec::new(),
        collision_pairs: Vec::new(),
        wind_coefficients: None,
        follow_allow_list: SlotConstruct::construct(&mut slot_values),
        vegetation: Vec::new(),
    })))
    .unwrap();
    let part = compile_model(
        &{
            part.import_values(&slot_values).unwrap();
            part.finish()
        }
        .unwrap(),
        |_| None,
    )
    .unwrap();
    let composed = host
        .compose([ModelPart::new(
            &part,
            PartBinding::Mounted(ModelMount::new(MountTarget::Node(carrier))),
        )])
        .unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = composed.prepare(&mut meshes, &mut binds);
    let mut materials = Assets::<ModelMaterial>::default();
    let mut app = App::new();
    app.add_systems(
        Update,
        (synchronize_mounted_models, animate_instances).chain(),
    );
    let mut queue = CommandQueue::default();
    let root = spawn_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    queue.apply(app.world_mut());
    app.insert_resource(materials);
    app.world_mut()
        .entity_mut(root)
        .insert(PhysicsSimulation::default());
    app.update();
    let child = app.world().get::<ModelRig>(root).unwrap().mounts()[0];
    let node = app
        .world()
        .get::<ModelRig>(child)
        .unwrap()
        .node(bone)
        .unwrap();
    app.world_mut()
        .get_mut::<PoseOverrides>(root)
        .unwrap()
        .0
        .insert(
            carrier,
            ModelTransform {
                rotation: Quat::from_array(
                    Quat::from_rotation_y(std::f32::consts::FRAC_PI_2).to_array(),
                ),
                ..SlotConstruct::construct(&mut slot_values)
            },
        );
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .shared_time = 0.1;
    app.world_mut()
        .get_mut::<Playback>(child)
        .unwrap()
        .shared_time = 0.1;
    app.update();
    let translation = app.world().get::<Transform>(node).unwrap().translation;
    assert!(
        translation.x > 0.01,
        "gravity did not follow current carrier rotation: {translation:?}"
    );
    assert!(
        translation.z.abs() < 1e-4,
        "inherited force used a stale carrier frame: {translation:?}"
    );
}

#[test]
fn world_frame_preserves_motion_while_model_frame_carries_bodies() {
    let mut slot_values = SlotValues::default();
    let (definition, authored) = fixture(1);
    for frame_mode in [PhysicsFrame::World, PhysicsFrame::Model] {
        let settings = PhysicsSimulation {
            gravity: Vec3::ZERO,
            frame: frame_mode,
            ..SlotConstruct::construct(&mut slot_values)
        };
        let mut state = PhysicsState::default();
        for frame in 0..=2 {
            let time = frame as f64 * 0.1;
            let basis = Affine3A::from_translation(Vec3::X * time as f32);
            state.rebase(&definition, &settings, time, Some(basis));
            state.step(&definition, &settings, time, &authored, &HashMap::new());
            let world_position = basis.transform_point3(translation(&state.output, 0));
            let expected = if frame_mode == PhysicsFrame::World {
                Vec3::ZERO
            } else {
                Vec3::X * time as f32
            };
            assert!((world_position - expected).length() < 1e-5);
        }
        let before = state.output[0];
        for _ in 0..100 {
            state.rebase(
                &definition,
                &settings,
                0.2,
                Some(Affine3A::from_translation(Vec3::X * 0.2)),
            );
            state.step(&definition, &settings, 0.2, &authored, &HashMap::new());
        }
        assert_eq!(before, state.output[0]);
        assert_eq!(state.bodies[0].velocity, Vec3::ZERO);
    }
}

#[test]
fn inertial_rotation_and_converted_forces_preserve_world_direction() {
    let mut slot_values = SlotValues::default();
    let (definition, mut authored) = fixture(1);
    authored[0] = Mat4::from_translation(Vec3::X * 2.);
    let mut settings = PhysicsSimulation {
        gravity: Vec3::NEG_Z * 9.81,
        acceleration: Vec3::Y,
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut state = PhysicsState::default();
    state.rebase(&definition, &settings, 0., Some(Affine3A::IDENTITY));
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    let basis = Affine3A::from_quat(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
    settings.gravity = basis.inverse().transform_vector3(settings.gravity);
    settings.acceleration = basis.inverse().transform_vector3(settings.acceleration);
    state.rebase(&definition, &settings, 0.1, Some(basis));
    state.step(&definition, &settings, 0.1, &authored, &HashMap::new());
    let world = basis.transform_point3(translation(&state.output, 0));
    assert!((world.x - 2.).abs() < 1e-4);
    assert!(world.z < -0.01 && world.y > 0.001);
}

#[test]
fn changing_sheared_frame_resets_inertial_state_to_animation() {
    let (definition, authored) = fixture(1);
    let settings = PhysicsSimulation::default();
    let mut state = PhysicsState::default();
    state.rebase(&definition, &settings, 0., Some(Affine3A::IDENTITY));
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    let shear = Affine3A::from_mat3_translation(
        Mat3::from_cols(Vec3::X, Vec3::new(0.5, 1., 0.), Vec3::Z),
        Vec3::ZERO,
    );
    state.rebase(&definition, &settings, 0.1, Some(shear));
    state.step(&definition, &settings, 0.1, &authored, &HashMap::new());
    assert_eq!(translation(&state.output, 0), Vec3::ZERO);
    assert!(state.frame_failure.is_some());
}

#[test]
fn ragdoll_and_private_world_equipment_preserve_domain_and_simulate_authored_chain() {
    let mut slot_values = SlotValues::default();
    use crate::compile_model;
    use crate::pose::{evaluate_pose, EvaluatedPose};
    use wc3::model::systems::model_physics::{CollisionMask, Motor, Physics};
    use wc3::model::values::model::PhysicsGameFlags;
    for domain in [PhysicsKind::Ragdoll, PhysicsKind::PrivateWorld] {
        let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
        let mut bodies = Vec::new();
        for index in 0..8 {
            let bone = builder
                .add_node(Node::named(format!("belt-bone-{index}"), &mut slot_values))
                .unwrap();
            let node = builder
                .add_node(
                    Node::named(format!("belt-body-{index}"), &mut slot_values).with_parent(bone),
                )
                .unwrap();
            bodies.push(Body {
                node,
                kind: if index == 0 || index == 7 {
                    PhysicsBodyKind::Kinematic
                } else {
                    PhysicsBodyKind::Dynamic
                },
                gravity_scale: 1.,
                inertia_scale: 1.,
                linear_damping: 0.1,
                angular_damping: 0.1,
                follow_factor: 0.,
                parent_body: None,
                attachment_host: index == 0,
                ragdoll_root: index == 0,
                shapes: vec![Shape {
                    transform: Mat4::IDENTITY,
                    geometry: ShapeGeometry::Capsule {
                        radius: 0.03,
                        endpoints: [
                            Vec3::from_array([0.05, 0., 0.]),
                            Vec3::from_array([0.05, 0., -0.19]),
                        ],
                    },
                    friction: 0.,
                    restitution: 0.,
                    density: 1.,
                    mass_properties: None,
                    game_flags: PhysicsGameFlags::default(),
                    collision_mask: CollisionMask::default(),
                }],
            });
        }
        let frame = Frame {
            axis_x: Vec3::from_array([-1., 0., 0.]),
            axis_y: Vec3::from_array([0., 1., 0.]),
            axis_z: Vec3::from_array([0., 0., -1.]),
            origin: Vec3::from_array([0.; 3]),
        };
        let joints = [(0, 1), (1, 2), (0, 3), (3, 4), (0, 5), (5, 6), (0, 7)]
            .into_iter()
            .enumerate()
            .map(|(index, (a, b))| Joint {
                body_a: bodies[a].node,
                body_b: bodies[b].node,
                frame_a: frame,
                frame_b: frame,
                constraint: if index == 6 {
                    Constraint::Weld {
                        linear: Spring::default(),
                        angular: Spring::default(),
                    }
                } else {
                    Constraint::Shoulder {
                        twist: [-0.25, 0.25],
                        cone: FRAC_PI_2,
                        motor: Motor::default(),
                    }
                },
            })
            .collect();
        builder
            .add_system(System::M2(ModelSystem::Physics(Physics {
                kind: domain,
                resource: None,
                bodies,
                joints,
                collision_pairs: Vec::new(),
                wind_coefficients: None,
                follow_allow_list: SlotConstruct::construct(&mut slot_values),
                vegetation: Vec::new(),
            })))
            .unwrap();
        let source = {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap();
        let compiled = compile_model(&source, |_| None).unwrap();
        assert_eq!(compiled.physics.bodies().count(), 8);
        assert_eq!(compiled.physics.joints().count(), 7);
        assert!(compiled.physics.bodies().all(|body| body.domain == domain));
        assert!(compiled
            .diagnostics
            .iter()
            .any(|entry| entry.message.contains("external-world collision")));
        let mut evaluated = vec![EvaluatedPose::default(); compiled.nodes.len()];
        for &index in compiled.order.iter() {
            let node = &compiled.nodes[index];
            evaluated[index] = evaluate_pose(
                node,
                node.rest,
                node.parent
                    .map_or_else(EvaluatedPose::default, |parent| evaluated[parent]),
                None,
            );
        }
        let authored: Vec<_> = evaluated.iter().map(|pose| pose.matrix).collect();
        let mut state = PhysicsState::default();
        for frame in 0..=24 {
            state.step(
                &compiled.physics,
                &PhysicsSimulation::default(),
                frame as f64 / 120.,
                &authored,
                &HashMap::new(),
            );
        }
        assert_eq!(state.output.iter().filter(|pose| pose.is_some()).count(), 6);
        assert!(state.output.iter().flatten().all(|pose| pose.is_finite()));
        assert!(
            state.output.iter().flatten().any(|pose| {
                let (_, rotation, _) = pose.to_scale_rotation_translation();
                rotation.angle_between(Quat::IDENTITY) > 0.01
            }),
            "authored-domain belt bodies did not respond to gravity"
        );
    }
}

#[test]
fn exact_capsules_remove_body_sphere_false_contacts() {
    let (mut definition, mut authored) = fixture(2);
    for body in &mut definition.bodies {
        body.fixtures = vec![Fixture::test_primitive(
            contact::Primitive::Capsule {
                endpoints: [-Vec3::Z * 2., Vec3::Z * 2.],
                radius: 0.1,
            },
            Vec3::ZERO,
            2.1,
            0.,
        )];
        body.radius = 2.1;
    }
    definition.pairs.push((0, 1));
    authored[1] = Mat4::from_translation(Vec3::X);
    let settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.step(
        &definition,
        &settings,
        1. / 120.,
        &authored,
        &HashMap::new(),
    );
    assert_eq!(state.bodies[0].position, Vec3::ZERO);
    assert_eq!(state.bodies[1].position, Vec3::X);
    assert_eq!(state.bodies[1].velocity, Vec3::ZERO);
}
#[test]
fn offset_capsule_impact_produces_angular_response_and_friction() {
    let (mut definition, mut authored) = fixture(2);
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    for body in &mut definition.bodies {
        body.fixtures = vec![Fixture::test_primitive(
            contact::Primitive::Capsule {
                endpoints: [-Vec3::Z, Vec3::Z],
                radius: 0.25,
            },
            Vec3::ZERO,
            1.25,
            0.8,
        )];
        body.radius = 1.25;
    }
    definition.pairs.push((0, 1));
    authored[1] = Mat4::from_translation(Vec3::new(0.49, 0., 0.8));
    let settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.bodies[1].velocity = Vec3::new(-1., 2., 0.);
    state.step(
        &definition,
        &settings,
        1. / 120.,
        &authored,
        &HashMap::new(),
    );
    assert!(state.bodies[1].angular_velocity.length() > 0.01);
    assert!(state.bodies[1].velocity.y < 2.);
    assert!(state.bodies[0].velocity.length() < 1e-6);
}
#[test]
fn moving_kinematic_compound_surface_drives_contact_friction() {
    let (mut definition, mut authored) = fixture(2);
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    for body in &mut definition.bodies {
        body.fixtures[0].friction = 0.8;
    }
    // The second fixture does not contact but belongs to the same moving compound.
    let mut offset = definition.bodies[0].fixtures[0].clone();
    offset.center = Vec3::X * 2.;
    offset.geometry = contact::Primitive::Sphere {
        center: offset.center,
        radius: 0.5,
    };
    definition.bodies[0].fixtures.push(offset);
    definition.bodies[0].radius = 2.5;
    definition.pairs.push((0, 1));
    authored[1] = Mat4::from_translation(Vec3::Z);
    let settings = PhysicsSimulation::default();
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    authored[0] = Mat4::from_translation(Vec3::Y * 0.01);
    state.step(
        &definition,
        &settings,
        1. / 120.,
        &authored,
        &HashMap::new(),
    );
    assert!((state.bodies[0].velocity.y - 1.2).abs() < 1e-5);
    assert!(state.bodies[1].velocity.y > 0.001);
    assert!(state.bodies[1].angular_velocity.x.abs() > 0.001);
}
#[test]
fn contact_repair_preserves_both_pose_histories() {
    let (mut definition, mut authored) = fixture(2);
    definition.pairs.push((0, 1));
    authored[1] = Mat4::from_translation(Vec3::X * 0.8);
    // Deliberately offset the fixtures to make normal repair angular.
    for body in &mut definition.bodies {
        body.fixtures[0].center = Vec3::Z * 0.3;
        body.fixtures[0].geometry = contact::Primitive::Sphere {
            center: Vec3::Z * 0.3,
            radius: 0.5,
        };
        body.radius = 0.8;
    }
    let settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.step(
        &definition,
        &settings,
        1. / 120.,
        &authored,
        &HashMap::new(),
    );
    for body in &state.bodies {
        assert!(body.velocity.length() < 1e-5);
        assert!(body.angular_velocity.length() < 1e-5);
    }
}

#[test]
fn moving_world_carrier_contributes_kinematic_contact_velocity() {
    let (mut definition, mut authored) = fixture(2);
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    for body in &mut definition.bodies {
        body.fixtures[0].friction = 0.8;
    }
    definition.pairs.push((0, 1));
    authored[1] = Mat4::from_translation(Vec3::Z);
    let settings = PhysicsSimulation::default();
    let mut state = PhysicsState::default();
    state.rebase(&definition, &settings, 0., Some(Affine3A::IDENTITY));
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.rebase(
        &definition,
        &settings,
        1. / 120.,
        Some(Affine3A::from_translation(Vec3::Y * 0.01)),
    );
    state.step(
        &definition,
        &settings,
        1. / 120.,
        &authored,
        &HashMap::new(),
    );
    assert!((state.bodies[0].velocity.y - 1.2).abs() < 1e-5);
    assert!(state.bodies[1].velocity.y > 0.001);
}
#[test]
fn paused_kinematic_target_motion_is_not_replayed_on_resume() {
    let (mut definition, mut authored) = fixture(1);
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    let mut settings = PhysicsSimulation::default();
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    settings.paused = true;
    authored[0] = Mat4::from_translation(Vec3::X * 2.);
    state.step(
        &definition,
        &settings,
        1. / 120.,
        &authored,
        &HashMap::new(),
    );
    settings.paused = false;
    state.step(
        &definition,
        &settings,
        2. / 120.,
        &authored,
        &HashMap::new(),
    );
    assert_eq!(state.bodies[0].position, Vec3::X * 2.);
    assert_eq!(state.bodies[0].velocity, Vec3::ZERO);
}

#[path = "admission_tests.rs"]
mod admission;

#[test]
fn contact_query_quota_is_shared_by_compounds_projections_and_substeps() {
    let (mut definition, mut authored) = fixture(2);
    // The body-level bound passes, but each tiny fixture pair is separated.
    // No projection can eliminate later work and mask a quota reset bug.
    for body in &mut definition.bodies {
        body.radius = 100.;
        body.fixtures = vec![
            Fixture::test_primitive(
                contact::Primitive::Sphere {
                    center: Vec3::ZERO,
                    radius: 0.01
                },
                Vec3::ZERO,
                0.01,
                0.
            );
            32
        ];
    }
    definition.pairs.push((0, 1));
    authored[1] = Mat4::from_translation(Vec3::X);
    let mut settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        contact_limits: PhysicsContactLimits {
            queries_per_update: 7,
            support_vertices_per_update: 0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.step(&definition, &settings, 1. / 60., &authored, &HashMap::new());
    assert_eq!(state.contact_stats().queries, 7);
    assert_eq!(state.contact_stats().query_budget_exhaustions, 1);
    assert_eq!(state.contact_stats().support_vertices, 0);
    state.step(&definition, &settings, 2. / 60., &authored, &HashMap::new());
    assert_eq!(state.contact_stats().queries, 14);
    assert_eq!(state.contact_stats().query_budget_exhaustions, 2);
    settings.paused = true;
    state.step(&definition, &settings, 3. / 60., &authored, &HashMap::new());
    assert_eq!(state.contact_stats().queries, 14);
    settings.reset();
    state.step(&definition, &settings, 4. / 60., &authored, &HashMap::new());
    assert_eq!(state.contact_stats(), PhysicsContactStats::default());
}
#[test]
fn zero_contact_quota_does_not_disable_body_integration_or_explicit_constraints() {
    let (mut definition, mut authored) = fixture(2);
    definition.pairs.push((0, 1));
    authored[1] = Mat4::from_translation(Vec3::X * 0.25);
    let settings = PhysicsSimulation {
        contact_limits: PhysicsContactLimits {
            queries_per_update: 0,
            support_vertices_per_update: 0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.step(
        &definition,
        &settings,
        1. / 120.,
        &authored,
        &HashMap::new(),
    );
    assert!(state.bodies[0].position.z < 0.);
    assert!((state.bodies[1].position.x - 0.25).abs() < 1e-6);
    assert_eq!(state.contact_stats().queries, 0);
    assert_eq!(state.contact_stats().query_budget_exhaustions, 1);
}

#[test]
fn explicitly_permitted_thin_hull_contacts_follow_the_face_not_its_sphere_bound() {
    use bevy::math::Vec4;
    let half = Vec3::new(1., 0.03, 0.4);
    let vertices = (0..8)
        .map(|bits| {
            Vec3::new(
                if bits & 1 == 0 { -half.x } else { half.x },
                if bits & 2 == 0 { -half.y } else { half.y },
                if bits & 4 == 0 { -half.z } else { half.z },
            )
        })
        .collect::<Vec<_>>();
    let shape = Shape {
        geometry: ShapeGeometry::ConvexHull {
            vertices: vertices.into_iter().collect(),
            planes: vec![
                Vec4::new(-1., 0., 0., half.x),
                Vec4::new(1., 0., 0., half.x),
                Vec4::new(0., -1., 0., half.y),
                Vec4::new(0., 1., 0., half.y),
                Vec4::new(0., 0., -1., half.z),
                Vec4::new(0., 0., 1., half.z),
            ]
            .into_iter()
            .collect(),
            faces: vec![
                vec![0, 4, 6, 2],
                vec![1, 3, 7, 5],
                vec![0, 1, 5, 4],
                vec![2, 6, 7, 3],
                vec![0, 2, 3, 1],
                vec![4, 5, 7, 6],
            ]
            .into(),
        },
        density: 1.,
        transform: Mat4::IDENTITY,
        friction: 0.,
        restitution: 0.,
        mass_properties: None,
        game_flags: Default::default(),
        collision_mask: Default::default(),
    };
    let (hull, _) = Fixture::prepare(&shape);
    assert!(hull.is_hull());
    assert!(hull.hull_issue().is_none());
    let (mut definition, mut authored) = fixture(2);
    definition.pairs.push((0, 1));
    definition.bodies[0].fixtures = vec![hull];
    definition.bodies[0].radius = half.length();
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    definition.bodies[1].fixtures = vec![Fixture::test_primitive(
        contact::Primitive::Sphere {
            center: Vec3::ZERO,
            radius: 0.1,
        },
        Vec3::ZERO,
        0.1,
        0.,
    )];
    definition.bodies[1].radius = 0.1;
    let settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        ..Default::default()
    };
    for initial_y in [0.3, 0.05] {
        authored[1] = Mat4::from_translation(Vec3::new(0.17, initial_y, 0.08));
        let mut state = PhysicsState::default();
        for frame in 0..=4 {
            state.step(
                &definition,
                &settings,
                f64::from(frame) / 120.,
                &authored,
                &HashMap::new(),
            );
        }
        let expected = if initial_y == 0.3 { initial_y } else { 0.13 };
        assert!(
            (state.bodies[1].position.y - expected).abs() < 2e-4,
            "{:?}",
            state.bodies[1].position
        );
        assert_eq!(state.bodies[0].position, Vec3::ZERO);
        assert_eq!(
            state.contact_stats().inconclusive_queries,
            0,
            "initial_y={initial_y}; matrix={:?}; hull_radius={}; {:?}",
            state.bodies[1].matrix(),
            definition.bodies[0].fixtures[0].radius,
            state.contact_stats()
        );
        assert!(state.contact_stats().queries > 0);
        assert!(state.contact_stats().support_vertices > 0);
    }
}

#[test]
fn separated_and_zero_weight_dense_pairs_share_one_examination_allowance() {
    for zero_weight in [false, true] {
        let (mut definition, mut authored) = fixture(8);
        for (i, body) in definition.bodies.iter_mut().enumerate() {
            authored[i] = if zero_weight {
                // Near/overlapping bounds cannot explain this rejection.
                body.inverse_mass = 0.;
                body.info.body.kind = PhysicsBodyKind::Kinematic;
                Mat4::IDENTITY
            } else {
                Mat4::from_translation(Vec3::X * (i as f32 * 10.))
            };
        }
        for a in 0..8 {
            for b in a + 1..8 {
                definition.pairs.push((a, b));
            }
        }
        let settings = PhysicsSimulation {
            iterations: 3,
            contact_limits: PhysicsContactLimits {
                pair_checks_per_update: 5,
                queries_per_update: 1000,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut state = PhysicsState::default();
        state.step(&definition, &settings, 0., &authored, &HashMap::new());
        state.step(&definition, &settings, 1. / 60., &authored, &HashMap::new());
        let stats = state.contact_stats();
        assert_eq!(
            stats.pair_checks, 5,
            "one allowance for every pass and substep"
        );
        assert_eq!(stats.pair_budget_exhaustions, 1);
        assert_eq!(
            stats.queries, 0,
            "charge BEFORE weight and broad-phase rejection"
        );
        assert_eq!(stats.support_vertices, 0);
        assert_eq!(stats.query_budget_exhaustions, 0);
        assert_eq!(stats.last_issue, Some(PhysicsContactIssue::PairLimit));
        if !zero_weight {
            assert!(state.bodies[0].position.z < 0.);
        }
        state.step(&definition, &settings, 2. / 60., &authored, &HashMap::new());
        assert_eq!(state.contact_stats().pair_checks, 10);
        assert_eq!(state.contact_stats().pair_budget_exhaustions, 2);
    }
}

#[test]
fn body_pair_budget_exact_boundary_zero_pause_and_explicit_reset() {
    for allowance in [0, 1, 3, 4, 5] {
        let (mut definition, mut authored) = fixture(2);
        definition.pairs.push((0, 1));
        authored[1] = Mat4::from_translation(Vec3::X * 10.);
        let mut settings = PhysicsSimulation {
            iterations: 1,
            contact_limits: PhysicsContactLimits {
                pair_checks_per_update: allowance,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut state = PhysicsState::default();
        state.step(&definition, &settings, 0., &authored, &HashMap::new());
        state.step(&definition, &settings, 1. / 60., &authored, &HashMap::new());
        // Two fixed substeps, each with one projection and one velocity pass.
        let stats = state.contact_stats();
        assert_eq!(stats.pair_checks, allowance.min(4) as u64);
        assert_eq!(stats.pair_budget_exhaustions, u64::from(allowance < 4));
        assert_eq!(stats.queries, 0);
        assert!(
            state.bodies[0].position.z < 0.,
            "contact quota must not pause forces"
        );
        state.step(&definition, &settings, 1. / 60., &authored, &HashMap::new());
        assert_eq!(
            state.contact_stats(),
            stats,
            "zero-delta update has no contact work"
        );
        settings.paused = true;
        state.step(&definition, &settings, 2. / 60., &authored, &HashMap::new());
        assert_eq!(state.contact_stats(), stats);
        settings.reset();
        state.step(&definition, &settings, 2. / 60., &authored, &HashMap::new());
        assert_eq!(state.contact_stats(), PhysicsContactStats::default());
    }
}

#[test]
fn zero_pair_allowance_keeps_joint_projection_active() {
    let (mut definition, mut authored) = fixture(2);
    authored[1] = Mat4::from_translation(Vec3::X * 2.);
    definition.bodies[0].inverse_mass = 0.;
    definition.bodies[0].info.body.kind = PhysicsBodyKind::Kinematic;
    let frame = Frame {
        axis_x: Vec3::X,
        axis_y: Vec3::Y,
        axis_z: Vec3::Z,
        origin: Vec3::ZERO,
    };
    definition.joints.push(JointDefinition {
        a: 0,
        b: 1,
        anchor_a: Vec3::ZERO,
        anchor_b: Vec3::ZERO,
        rotation_a: Quat::IDENTITY,
        rotation_b: Quat::IDENTITY,
        authored: Joint {
            body_a: definition.bodies[0].info.body.node,
            body_b: definition.bodies[1].info.body.node,
            frame_a: frame,
            frame_b: frame,
            constraint: Constraint::Distance {
                distance: 1.,
                spring: Spring::default(),
            },
        },
    });
    definition.pairs.push((0, 1));
    let settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        contact_limits: PhysicsContactLimits {
            pair_checks_per_update: 0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    state.step(
        &definition,
        &settings,
        1. / 120.,
        &authored,
        &HashMap::new(),
    );
    assert!((state.bodies[0].position.distance(state.bodies[1].position) - 1.).abs() < 1e-5);
    assert_eq!(state.contact_stats().pair_checks, 0);
    assert_eq!(state.contact_stats().queries, 0);
    assert_eq!(state.contact_stats().pair_budget_exhaustions, 1);
}

#[test]
fn unpaired_ballistic_motion_matches_independent_discrete_oracle_across_frame_slices() {
    let (definition, authored) = fixture(1);
    let settings = PhysicsSimulation {
        gravity: Vec3::new(0., 0., -0.4),
        timestep: 1. / 120.,
        ..Default::default()
    };
    let steps = 240;
    let acceleration = f64::from(settings.gravity.z);
    let h = settings.timestep;
    // Semi-implicit Euler under constant acceleration, derived independently:
    // v_n=n*a*h and x_n=a*h²*n*(n+1)/2, with initial x=v=0.
    let expected_velocity = acceleration * h * f64::from(steps);
    let expected_position = acceleration * h * h * f64::from(steps * (steps + 1)) / 2.;
    let mut results = Vec::new();
    for frame_steps in [1, 4] {
        let mut state = PhysicsState::default();
        state.step(&definition, &settings, 0., &authored, &HashMap::new());
        for completed in (frame_steps..=steps).step_by(frame_steps as usize) {
            state.step(
                &definition,
                &settings,
                f64::from(completed) * h,
                &authored,
                &HashMap::new(),
            );
        }
        assert!(
            (f64::from(state.bodies[0].position.z) - expected_position).abs() < 5e-5,
            "position {:?}; oracle={expected_position}",
            state.bodies[0].position
        );
        assert!((f64::from(state.bodies[0].velocity.z) - expected_velocity).abs() < 1e-5);
        assert_eq!(state.contact_stats(), PhysicsContactStats::default());
        results.push((state.bodies[0].position, state.bodies[0].velocity));
    }
    assert_eq!(
        results[0], results[1],
        "frame slicing must preserve the same fixed steps"
    );
}

#[test]
fn long_ballistic_flight_does_not_feed_large_position_quantization_into_velocity() {
    let (definition, mut authored) = fixture(1);
    authored[0] = Mat4::from_translation(Vec3::Z * 65_536.);
    let settings = PhysicsSimulation {
        gravity: Vec3::new(0., 0., -0.25),
        timestep: 1. / 128.,
        ..Default::default()
    };
    let steps = 4096;
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    for completed in 1..=steps {
        state.step(
            &definition,
            &settings,
            f64::from(completed) * settings.timestep,
            &authored,
            &HashMap::new(),
        );
    }
    // Binary-exact acceleration increments give an independent exact velocity.
    let expected_velocity = -0.25 * settings.timestep * f64::from(steps);
    assert_eq!(f64::from(state.bodies[0].velocity.z), expected_velocity);
    let expected_position = 65_536.
        - 0.25 * settings.timestep * settings.timestep * f64::from(steps * (steps + 1)) / 2.;
    // Position remains f32: bound its accumulated rounding by N*half-ULP at
    // the largest position. This does not promise f64 trajectory precision.
    let position_error_bound = f64::from(steps) * f64::from(65_536_f32.next_up() - 65_536.) / 2.;
    assert!(
        (f64::from(state.bodies[0].position.z) - expected_position).abs() <= position_error_bound
    );
    assert!(state.bodies[0].position.z < 65_436.);
}

#[test]
fn moving_contact_repair_preserves_integrated_linear_momentum() {
    let (mut definition, mut authored) = fixture(2);
    authored[1] = Mat4::from_translation(Vec3::X * 0.8);
    definition.pairs.push((0, 1));
    let settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        timestep: 1. / 128.,
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    for body in &mut state.bodies {
        body.velocity = Vec3::Y * 2.;
    }
    state.step(
        &definition,
        &settings,
        settings.timestep,
        &authored,
        &HashMap::new(),
    );
    assert!((state.bodies[1].position - state.bodies[0].position).length() >= 1. - 1e-5);
    for body in &state.bodies {
        assert!(body.velocity.distance(Vec3::Y * 2.) < 1e-5);
    }
    assert!((state.bodies[0].velocity + state.bodies[1].velocity).distance(Vec3::Y * 4.) < 1e-5);
}

#[test]
fn joint_projection_still_contributes_constraint_velocity_without_losing_translation() {
    let (mut definition, mut authored) = fixture(2);
    authored[1] = Mat4::from_translation(Vec3::X * 2.);
    let frame = Frame {
        axis_x: Vec3::X,
        axis_y: Vec3::Y,
        axis_z: Vec3::Z,
        origin: Vec3::ZERO,
    };
    definition.joints.push(JointDefinition {
        a: 0,
        b: 1,
        anchor_a: Vec3::ZERO,
        anchor_b: Vec3::ZERO,
        rotation_a: Quat::IDENTITY,
        rotation_b: Quat::IDENTITY,
        authored: Joint {
            body_a: definition.bodies[0].info.body.node,
            body_b: definition.bodies[1].info.body.node,
            frame_a: frame,
            frame_b: frame,
            constraint: Constraint::Distance {
                distance: 1.,
                spring: Spring::default(),
            },
        },
    });
    let settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        timestep: 1. / 128.,
        iterations: 1,
        ..Default::default()
    };
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    for body in &mut state.bodies {
        body.velocity = Vec3::Y * 2.;
    }
    state.step(
        &definition,
        &settings,
        settings.timestep,
        &authored,
        &HashMap::new(),
    );
    assert!((state.bodies[1].position.distance(state.bodies[0].position) - 1.).abs() < 1e-6);
    // Equal mass, central anchors: the unit distance correction is split equally.
    assert!(state.bodies[0].velocity.distance(Vec3::new(64., 2., 0.)) < 1e-5);
    assert!(state.bodies[1].velocity.distance(Vec3::new(-64., 2., 0.)) < 1e-5);
    assert!((state.bodies[0].velocity + state.bodies[1].velocity).distance(Vec3::Y * 4.) < 1e-5);
}

#[test]
fn selected_primitive_pairs_prepare_scratch_before_scale_animation_needs_it() {
    let (mut definition, mut authored) = fixture(2);
    authored[1] = Mat4::from_translation(Vec3::X * 0.8);
    let mut settings = PhysicsSimulation {
        gravity: Vec3::ZERO,
        iterations: 1,
        ..Default::default()
    };
    let mut no_contacts = PhysicsState::default();
    no_contacts.step(&definition, &settings, 0., &authored, &HashMap::new());
    assert!(no_contacts.contact_scratch.is_none());
    definition.pairs.push((0, 1));
    let mut state = PhysicsState::default();
    state.step(&definition, &settings, 0., &authored, &HashMap::new());
    let scratch = state.contact_scratch.as_deref().unwrap() as *const ContactScratch;
    // Uniform spheres initially use the analytical path. A later body scale
    // makes one an ellipsoid, requiring support without a first-query allocation.
    authored[1] = Mat4::from_scale_rotation_translation(
        Vec3::new(0.5, 1., 1.),
        Quat::IDENTITY,
        Vec3::X * 0.8,
    );
    state.step(
        &definition,
        &settings,
        settings.timestep,
        &authored,
        &HashMap::new(),
    );
    assert_eq!(
        state.contact_scratch.as_deref().unwrap() as *const ContactScratch,
        scratch
    );
    assert!(state.contact_stats().support_vertices > 0);
    assert_eq!(state.contact_stats().inconclusive_queries, 0);
    // This independently prepared fixture has no inferred pairs. Switching to
    // that empty selection retains the already owned scratch for later reuse.
    settings.contact_policy = PhysicsContactPolicy::UnconnectedDynamics;
    state.step(
        &definition,
        &settings,
        settings.timestep,
        &authored,
        &HashMap::new(),
    );
    assert_eq!(
        state.contact_scratch.as_deref().unwrap() as *const ContactScratch,
        scratch
    );
}
