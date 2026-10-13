use wc3::model::{
    conversion::{self, Asset, ExportOptions, ImportOptions},
    m3::{self, records as nr},
    wem::{self, records as wr},
};
const IDENTITY: [[f32; 4]; 4] = [
    [1., 0., 0., 0.],
    [0., 1., 0., 0.],
    [0., 0., 1., 0.],
    [0., 0., 0., 1.],
];
fn source() -> m3::Bundle<'static, m3::Md34> {
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
    let unused = doc.append(&[nr::Byte { value: 73 }]).unwrap();
    let mut shape_record = nr::PhysicsShape::<nr::V1> {
        transform: IDENTITY,
        shape_type: 1,
        shape_dimensions: [2., 0., 0.],
        planes: m3::Ref::from_section(1, planes.index()),
        unused: m3::Ref::from_section(1, unused.index()),
        ..Default::default()
    };
    shape_record.legacy_reserved_0.0 = [1, 2, 3, 4];
    shape_record.legacy_reserved_1.0 = [5, 6, 7];
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
                    flags: 3,
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
            joint_type: 1,
            matrix_body1: IDENTITY,
            matrix_body2: IDENTITY,
            enable_limits: 0x101,
            limit_min: -1.,
            limit_max: 1.,
            enable_friction: 0x201,
            friction: 0.125,
            angular_frequency: 6.,
            break_threshold: 42.,
            enable_shape: 1,
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
            vertex_flags: 0x0182_007d,
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
fn physics_and_attachment_fields_are_active_and_editable() {
    let imported = conversion::m3::import(&source(), &ImportOptions::default()).unwrap();
    let mut raw = imported.value.document.into_raw();
    let root = raw
        .resolve::<wr::DocumentRecord<wr::V1>>(wem::Ref::from_section(
            1,
            raw.root_reference().index,
        ))
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let model = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let body_id = raw.section_id(model.physics.bodies.index).unwrap();
    let mut bodies = raw
        .array::<wr::PhysicsBody<wr::V3>>(body_id)
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(bodies.len(), 2);
    bodies[0].linear_damping = 0.125;
    bodies[0].gravity_scale = 0.5;
    raw.replace(body_id, &bodies).unwrap();
    let asset = Asset::new(wem::Document::from_raw(raw).unwrap());
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
    let shape = output
        .model
        .document()
        .resolve::<nr::PhysicsShape<nr::V1>>(body.rigid_body_shape.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(shape.legacy_reserved_0.0, [1, 2, 3, 4]);
    assert_eq!(shape.legacy_reserved_1.0, [5, 6, 7]);
    assert_eq!(
        output
            .model
            .document()
            .resolve::<nr::Byte>(shape.unused.cast())
            .unwrap()
            .get(0)
            .unwrap()
            .unwrap()
            .value,
        73
    );
    assert_eq!(
        output
            .model
            .document()
            .resolve::<nr::Vector4>(shape.planes.cast())
            .unwrap()
            .get(0)
            .unwrap()
            .unwrap()
            .value,
        [0., 0., 1., 2.]
    );
    assert_eq!(body.mass, 3.);
    assert_eq!(body.gravity_factor, 0.5);
    assert_eq!(body.linear_damping, 0.125);
    assert_eq!(body.priority, 7);
    assert_eq!(body.flags, 3);
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
    assert_eq!(joint.angular_frequency, 6.);
    assert_eq!(joint.break_threshold, 42.);
    let attachment = output
        .model
        .document()
        .resolve::<nr::AttachmentPoint<nr::V1>>(root.attachment_points.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(attachment.bone_index, 1);
    assert_eq!(attachment.unknown, 123);
}
