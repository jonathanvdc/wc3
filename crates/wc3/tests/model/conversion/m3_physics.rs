use wc3::model::{
    conversion::{self, ExportOptions, ImportOptions, LossPolicy},
    m3::{self, records as nr},
    semantic::profiles::m3 as surface,
};
const IDENTITY: [[f32; 4]; 4] = [
    [1., 0., 0., 0.],
    [0., 1., 0., 0.],
    [0., 0., 1., 0.],
    [0., 0., 0., 1.],
];
fn source() -> m3::Bundle<'static, m3::Md34> {
    source_with_unresolved_shape_bytes(false)
}
fn source_with_unresolved_shape_bytes(populated: bool) -> m3::Bundle<'static, m3::Md34> {
    let mut doc = m3::Document::<m3::Md34>::new();
    let bones = doc
        .append(
            &[nr::Bone::<nr::V1> {
                parent_index: u16::MAX,
                rotation: nr::AnimationValue {
                    initial_value: [0., 0., 0., 1.],
                    ..Default::default()
                },
                scale: nr::AnimationValue {
                    initial_value: [1.; 3],
                    ..Default::default()
                },
                ..Default::default()
            }; 2],
        )
        .unwrap();
    let planes = doc
        .append(&[nr::Vector4 {
            value: [0., 0., 1., 2.],
        }])
        .unwrap();

    let shape_record = nr::PhysicsShape::<nr::V1> {
        transform: IDENTITY,
        shape_type: nr::PhysicsShapeKind::Sphere,
        shape_dimensions: [2., 0., 0.],
        planes: m3::Ref::from_section(1, planes.index()),
        legacy_reserved_1: nr::FixedBytes(if populated { [1, 0, 0] } else { [0; 3] }),

        ..Default::default()
    };

    let shape = doc.append(&[shape_record]).unwrap();
    let bodies = doc
        .append(
            &(0..2)
                .map(|bone| nr::RigidBody::<nr::V4> {
                    parent_bone_index: bone,
                    simulation_type: 0,
                    mass: 3.,
                    friction: 0.5,
                    restitution: 0.25,
                    gravity_factor: 1.,
                    flags: nr::RigidBodyFlags(3),
                    priority: 7,
                    rigid_body_shape: m3::Ref::from_section(1, shape.index()),
                    dynamic_state: nr::AnimationValue {
                        initial_value: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let joints = doc
        .append(&[nr::PhysicsJoint {
            bone_index1: 0,
            bone_index2: 1,
            joint_type: nr::JointKind::Revolute,
            matrix_body1: IDENTITY,
            matrix_body2: IDENTITY,
            enable_limits: 0x101,
            limit_min: -1.,
            limit_max: 1.,
            enable_friction: 0x201,
            friction: 0.125,
            angular_frequency: 6.,
            break_threshold: 42.,
            collision_control: 1,
            ..Default::default()
        }])
        .unwrap();
    let name = doc
        .append(
            &b"socket\0"
                .iter()
                .map(|&value| nr::Character { value })
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let attachments = doc
        .append(&[nr::AttachmentPoint::<nr::V1> {
            unknown: 123,
            name: m3::Ref::from_section(7, name.index()),
            bone_index: 1,
        }])
        .unwrap();
    let root = doc
        .append(&[nr::ModelRecord::<nr::V29> {
            vertex_flags: nr::VertexFlags(0x0182_007d),
            bones: m3::Ref::from_section(2, bones.index()),
            rigid_bodies: m3::Ref::from_section(2, bodies.index()),
            physics_joints: m3::Ref::from_section(1, joints.index()),
            attachment_points: m3::Ref::from_section(1, attachments.index()),
            ..Default::default()
        }])
        .unwrap();
    doc.set_root(m3::Ref::<nr::ModelRecord<nr::V29>>::from_section(
        1,
        root.index(),
    ));
    m3::Bundle::from_model(m3::Model::from_document(doc).unwrap())
}

#[test]
fn unresolved_shape_bytes_require_explicit_loss_policy() {
    let source = source_with_unresolved_shape_bytes(true);
    let error = conversion::m3::import(&source, &ImportOptions::default()).unwrap_err();
    assert!(error
        .to_string()
        .contains("m3.physics.shape.legacy_reserved_1"));
    let result = conversion::m3::import(
        &source,
        &ImportOptions {
            loss_policy: LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(result
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "m3.physics.shape.legacy_reserved_1"));
}
#[test]
fn physics_and_attachment_fields_are_active_and_editable() {
    let source = source();
    let mut asset = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, system) = asset.models[0]
        .systems()
        .find(|(_, system)| matches!(system, surface::System::Physics(_)))
        .unwrap();
    let mut system = system.clone();
    let surface::System::Physics(physics) = &mut system else {
        unreachable!()
    };
    assert_eq!(physics.bodies.len(), 2);
    let params = physics.bodies[0].parameters.as_mut().unwrap();
    params.linear_damping = 0.125;
    params.gravity_factor = 0.5;
    let surface::ShapeGeometry::Sphere { radius, .. } = &mut physics.bodies[0].shapes[0].geometry
    else {
        panic!("sphere")
    };
    *radius = 3.;
    asset
        .edit_model(0, |builder| builder.set_system(id, system))
        .unwrap();
    let output = conversion::m3::plan::<m3::StarCraft2>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    assert_eq!(root.bones.entries, 2);
    let body = output
        .model
        .document()
        .resolve::<nr::RigidBody<nr::V4>>(root.rigid_bodies.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(body.linear_damping, 0.125);
    assert_eq!(body.gravity_factor, 0.5);
    assert_eq!(body.mass, 3.);
    let shape = output
        .model
        .document()
        .resolve::<nr::PhysicsShape<nr::V1>>(body.rigid_body_shape.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(shape.shape_dimensions, [3., 0., 0.]);
    let planes = output
        .model
        .document()
        .resolve::<nr::Vector4>(shape.planes.cast())
        .unwrap();
    assert_eq!(planes.get(0).unwrap().unwrap().value, [0., 0., 1., 2.]);
    let joint = output
        .model
        .document()
        .resolve::<nr::PhysicsJoint>(root.physics_joints.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(joint.enable_limits, 0x101);
    assert_eq!(joint.enable_friction, 0x201);
    assert_eq!(joint.friction, 0.125);
    assert_eq!(joint.break_threshold, 42.);
    let attachment = output
        .model
        .document()
        .resolve::<nr::AttachmentPoint<nr::V1>>(root.attachment_points.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(attachment.unknown, 123);
    assert_eq!(attachment.bone_index, 1);
    let reread = conversion::m3::import(&output, &ImportOptions::default()).unwrap();
    assert!(reread.value.models[0].features().any(|(_,feature)|matches!(&feature.payload,surface::Feature::Attachment(attachment)if attachment.name=="socket")));
}

#[test]
fn selection_and_attachment_volumes_use_editable_collision_meshes() {
    let source = source();
    let root_id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = source.model.into_document();
    let points = document
        .append(&[
            nr::Vector3 {
                value: [0., 0., 0.],
            },
            nr::Vector3 {
                value: [1., 0., 0.],
            },
            nr::Vector3 {
                value: [0., 1., 0.],
            },
        ])
        .unwrap();
    let corners = document
        .append(&[
            nr::U16 { value: 0 },
            nr::U16 { value: 1 },
            nr::U16 { value: 2 },
        ])
        .unwrap();
    root.tight_hit_test_object = nr::HitTestShape {
        shape_type: nr::VolumeShape::Box,
        bone_index: 1,
        transform: IDENTITY,
        size_x: 3.,
        size_y: 4.,
        size_z: 5.,
        ..Default::default()
    };
    let fuzzy = document
        .append(&[nr::HitTestShape::<nr::V0> {
            shape_type: nr::VolumeShape::Mesh,
            bone_index: 0,
            transform: IDENTITY,
            vertex_positions: m3::Ref::from_section(3, points.index()),
            face_indices: m3::Ref::from_section(3, corners.index()),
            ..Default::default()
        }])
        .unwrap();
    root.fuzzy_hit_test_objects = m3::Ref::from_section(1, fuzzy.index());
    let attachment = document
        .append(&[nr::AttachmentVolume {
            shape_type: nr::VolumeShape::Sphere,
            bone_index: 0,
            bone1: 0,
            bone2: 1,
            transform: IDENTITY,
            size_x: 2.,
            ..Default::default()
        }])
        .unwrap();
    root.attachment_volumes = m3::Ref::from_section(1, attachment.index());
    document.replace(root_id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let mut asset = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, original) = asset.models[0].features().find(|(_, value)| matches!(&value.payload, surface::Feature::HitTest(shape) if shape.role == surface::HitTestRole::Fuzzy)).unwrap();
    let mut edited = original.clone();
    let surface::Feature::HitTest(shape) = &mut edited.payload else {
        unreachable!()
    };
    let mesh = asset.models[0].mesh(shape.mesh.unwrap()).unwrap();
    assert!(mesh.positions().is_borrowed());
    assert_eq!(mesh.positions().get(1), Some([1., 0., 0.]));
    shape.size_x = 7.;
    asset
        .edit_model(0, |builder| builder.set_feature(id, edited.payload))
        .unwrap();
    let output = conversion::m3::plan::<m3::StarCraft2>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    assert_eq!(root.tight_hit_test_object.size_x, 3.);
    assert_eq!(root.tight_hit_test_object.bone_index, 1);
    let fuzzy = output
        .model
        .document()
        .resolve::<nr::HitTestShape<nr::V1>>(root.fuzzy_hit_test_objects.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(fuzzy.size_x, 7.);
    assert_eq!(fuzzy.face_indices.entries, 3);
    let attachment = output
        .model
        .document()
        .resolve::<nr::AttachmentVolume>(root.attachment_volumes.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(attachment.bone2, 1);
    assert_eq!(attachment.size_x, 2.);
}

#[test]
fn named_turrets_reference_active_parts_instead_of_trigger_data() {
    let source = source();
    let root_id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = source.model.into_document();
    let part = document
        .append(&[nr::TurretBehavior::<nr::V1> {
            transform: IDENTITY,
            bone_index: 1,
            use_as_main_turret: nr::TurretPartFlags(0x81),
            yaw_limited: nr::TurretAxisFlags(0x8000_0002),
            yaw_min: -1.,
            yaw_max: 1.,
            unknown3: 1.,
            unknown4: 1.,
            ..Default::default()
        }])
        .unwrap();
    let members = document.append(&[nr::U32 { value: 0 }]).unwrap();
    let name = document
        .append(&[
            nr::Character { value: b'a' },
            nr::Character { value: b'i' },
            nr::Character { value: b'm' },
            nr::Character { value: 0 },
        ])
        .unwrap();
    let groups = document
        .append(&[nr::TurretGroup {
            parts: m3::Ref::from_section(1, members.index()),
            name: m3::Ref::from_section(4, name.index()),
        }])
        .unwrap();
    root.turret_behaviors = m3::Ref::from_section(1, part.index());
    root.turret_groups = m3::Ref::from_section(1, groups.index());
    document.replace(root_id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let mut asset = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let groups = asset.models[0]
        .systems()
        .find_map(|(_, system)| match system {
            surface::System::TurretGroups(groups) => Some(groups),
            _ => None,
        })
        .unwrap();
    assert_eq!(groups[0].name, "aim");
    assert!(matches!(groups[0].name, std::borrow::Cow::Borrowed(_)));
    let part_id = groups[0].parts[0];
    let mut part = asset.models[0].feature(part_id).unwrap().payload.clone();
    let surface::Feature::Turret(value) = &mut part else {
        unreachable!()
    };
    assert!(value.yaw_limited.limited());
    value.yaw_max = 2.;
    asset
        .edit_model(0, |builder| builder.set_feature(part_id, part))
        .unwrap();
    let output = conversion::m3::plan::<m3::StarCraft2>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let part = output
        .model
        .document()
        .resolve::<nr::TurretBehavior<nr::V4>>(root.turret_behaviors.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(part.yaw_max, 2.);
    assert_eq!(part.yaw_limited.bits(), 0x8000_0002);
    assert_eq!(part.use_as_main_turret.bits(), 0x81);
    let group = output
        .model
        .document()
        .resolve::<nr::TurretGroup>(root.turret_groups.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let member = output
        .model
        .document()
        .resolve::<nr::U32>(group.parts.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(member.value, 0);
}

#[test]
fn body_constraints_use_checked_local_identities() {
    let source = source();
    let root_id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = source.model.into_document();
    let dependent = document.append(&[nr::U16 { value: 1 }]).unwrap();
    let constraint = document
        .append(&[nr::PhysicsConstraint {
            rigid_body_1: 0,
            rigid_body_2: 1,
            dependents: m3::Ref::from_section(1, dependent.index()),
            flags: nr::ConstraintFlags(0x8123_4567),
            break_force: 42.,
        }])
        .unwrap();
    root.physics_constraints = m3::Ref::from_section(1, constraint.index());
    document.replace(root_id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let mut asset = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, system) = asset.models[0]
        .systems()
        .find(|(_, system)| matches!(system, surface::System::Physics(_)))
        .unwrap();
    let mut edited = system.clone();
    let surface::System::Physics(world) = &mut edited else {
        unreachable!()
    };
    assert_eq!(world.constraints[0].body_b, world.bodies[1].id());
    assert_eq!(world.constraints[0].dependents[0], world.bodies[1].id());
    world.constraints[0].break_force = 75.;
    asset
        .edit_model(0, |builder| builder.set_system(id, edited))
        .unwrap();
    let output = conversion::m3::plan::<m3::StarCraft2>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let row = output
        .model
        .document()
        .resolve::<nr::PhysicsConstraint>(root.physics_constraints.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(row.break_force, 75.);
    assert_eq!(row.flags.bits(), 0x8123_4567);
    assert_eq!(row.rigid_body_2, 1);
    let mut invalid = asset.models[0].system(id).unwrap().clone();
    let surface::System::Physics(world) = &mut invalid else {
        unreachable!()
    };
    let foreign = surface::Body::new(world.bodies[0].node);
    world.constraints[0].body_b = foreign.id();
    assert!(asset
        .edit_model(0, |builder| builder.set_system(id, invalid))
        .is_err());
    let surface::System::Physics(world) = asset.models[0].system(id).unwrap() else {
        unreachable!()
    };
    assert_eq!(world.bodies.len(), 2);
}

#[test]
fn animation_split_declarations_preserve_unresolved_typed_controls() {
    let source = source();
    let root_id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = source.model.into_document();
    let items = document
        .append(&[nr::U16 { value: 17 }, nr::U16 { value: u16::MAX }])
        .unwrap();
    let declarations = document
        .append(&[nr::BoneAnimationSet {
            flags: nr::BoneSetFlags(0x8000_1234),
            animation_sequence_index: 32769,
            fallback_sequence_index: u16::MAX,
            split_items: m3::Ref::from_section(2, items.index()),
            ..Default::default()
        }])
        .unwrap();
    root.bone_animation_sets = m3::Ref::from_section(1, declarations.index());
    root.animation_split_count = 0x8123_4567;
    document.replace(root_id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let asset = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let items = asset.models[0]
        .systems()
        .find_map(|(_, system)| match system {
            surface::System::AnimationPartitions(declarations) => {
                Some(&declarations[0].split_items)
            }
            _ => None,
        })
        .unwrap();
    assert!(items.is_borrowed());
    assert_eq!(items.get(1), Some(u16::MAX));
    let output = conversion::m3::plan::<m3::StarCraft2>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    assert_eq!(root.animation_split_count, 0x8123_4567);
    let declaration = output
        .model
        .document()
        .resolve::<nr::BoneAnimationSet>(root.bone_animation_sets.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(declaration.animation_sequence_index, 32769);
    assert_eq!(declaration.fallback_sequence_index, u16::MAX);
    assert_eq!(declaration.flags.bits(), 0x8000_1234);
}

#[test]
fn unresolved_reference_qualifiers_require_explicit_loss_policy() {
    let source = source();
    let root_id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    root.bones.flags = nr::ReferenceFlags(0x8000_0001);
    let mut document = source.model.into_document();
    document.replace(root_id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let error = conversion::m3::import(&source, &ImportOptions::default()).unwrap_err();
    assert!(error.to_string().contains("m3.reference_qualifier.MODL"));
    let converted = conversion::m3::import(
        &source,
        &ImportOptions {
            loss_policy: LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(converted
        .report
        .issues
        .iter()
        .any(|issue| issue.path.starts_with("m3.reference_qualifier.MODL")));
    assert_eq!(converted.value.models[0].nodes().count(), 2);
}
