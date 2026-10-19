use super::*;
use std::f64::consts::FRAC_PI_2;
use wc3::model::systems::model_physics::Spring;
use wc3::model::ModelMetadata;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{ModelMount, MountTarget};
use wc3::model::{Node, Quaternion};

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
            inverse_inertia: 1.,
            radius: 0.5,
            center: Vec3::ZERO,
            restitution: 0.,
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
                axis_x: Vec3::X.to_array(),
                axis_y: Vec3::Y.to_array(),
                axis_z: Vec3::Z.to_array(),
                origin: [0.; 3],
            },
            frame_b: Frame {
                axis_x: Vec3::X.to_array(),
                axis_y: Vec3::Y.to_array(),
                axis_z: Vec3::Z.to_array(),
                origin: [0.; 3],
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
                        translation: [0., 0., 1.],
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
                translation: [0., 0., 3.],
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
        axis_x: Vec3::X.to_array(),
        axis_y: Vec3::Y.to_array(),
        axis_z: Vec3::Z.to_array(),
        origin: [0.; 3],
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
            translation: [x, 0., 0.],
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
    definition.bodies[1].center = Vec3::new(1., 0., -1.);
    authored[1] = Mat4::IDENTITY;
    let frame = Frame {
        axis_x: Vec3::X.to_array(),
        axis_y: Vec3::Y.to_array(),
        axis_z: Vec3::Z.to_array(),
        origin: [0.; 3],
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
                rotation: Quaternion(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2).to_array()),
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
                    transform: Mat4::IDENTITY.to_cols_array_2d(),
                    geometry: ShapeGeometry::Capsule {
                        radius: 0.03,
                        endpoints: [[0.05, 0., 0.], [0.05, 0., -0.19]],
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
            axis_x: [-1., 0., 0.],
            axis_y: [0., 1., 0.],
            axis_z: [0., 0., -1.],
            origin: [0.; 3],
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
