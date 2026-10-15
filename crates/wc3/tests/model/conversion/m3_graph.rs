use wc3::model::common::IntoZeroVec;
use wc3::model::io;
use wc3::model::Model;
use wc3::model::{
    formats::m3::{self, records as nr},
    profiles::m3 as surface,
    profiles::M3,
};

fn source() -> m3::Bundle<'static> {
    let model = m3::Document::new::<m3::StarCraft2>().unwrap();
    let id = model.root_id();
    let mut root = model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = model.into_container();
    let name = document
        .append(&[nr::Character { value: b'b' }, nr::Character { value: 0 }])
        .unwrap();
    let bone = nr::Bone::<nr::V1> {
        id: 17,
        name: m3::Ref::from_section(2, name.index()),
        rotation: nr::AnimationValue {
            initial_value: [0., 0., 0., 1.],
            null_value: [0., 0., 0., 1.],
            ..Default::default()
        },
        scale: nr::AnimationValue {
            initial_value: [1.; 3],
            null_value: [1.; 3],
            ..Default::default()
        },
        parent_index: u16::MAX,
        ..Default::default()
    };
    let bones = document.append(&[bone]).unwrap();
    root.bones = m3::Ref::from_section(1, bones.index());
    document.replace(id, &[root]).unwrap();
    m3::Bundle::from_model(m3::Document::from_container(document).unwrap())
}
#[test]
fn imported_typed_graph_edits_rebuild_native_bone_fields() {
    let source = source();
    let mut imported: Model<'_, M3> = io::m3::import(&source, &Default::default()).unwrap().value;
    let (id, node) = imported.nodes().find(|(_, node)| node.name == "b").unwrap();
    assert!(matches!(node.name, std::borrow::Cow::Borrowed(_)));
    let mut node = node.clone();
    node.transform.translation = [3., 4., 5.];
    let surface::NodeRole::Bone(controls) = &mut node.controls else {
        panic!("expected actual bone")
    };
    controls.batching.initial = 0x8000_0001;
    imported.edit(|draft| draft.set_node(id, node)).unwrap();
    let output = io::m3::plan::<m3::StarCraft2>(&imported, &Default::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root_record().unwrap();
    let bones = output
        .model
        .container()
        .resolve::<nr::Bone<nr::V1>>(root.bones().cast())
        .unwrap();
    let bone = bones.get(0).unwrap().unwrap();
    assert_eq!(bone.position.initial_value, [3., 4., 5.]);
    assert_eq!(bone.batching.initial_value, 0x8000_0001);
    assert_eq!(bone.id, 17);
    let reread = io::m3::import(&output, &Default::default()).unwrap();
    let (_, node) = reread
        .value
        .nodes()
        .find(|(_, node)| node.name == "b")
        .unwrap();
    assert_eq!(node.transform.translation, [3., 4., 5.]);
}
#[test]
fn complete_camera_optics_rebuild_without_editable_document_backing() {
    let source = source();
    let mut imported = io::m3::import(&source, &Default::default()).unwrap().value;
    let (id, _) = imported.nodes().find(|(_, node)| node.name == "b").unwrap();
    let constant = |value| surface::Property {
        initial: value,
        reset: value,
        control: surface::AnimationControl::default(),
        binding: None,
    };
    let camera = surface::Camera {
        name: "Main camera".into(),
        field_of_view: constant(1.),
        use_vertical_f_o_v: 0,
        optics: surface::CameraOptics::Legacy { controls: [2., 3.] },
        shadow_clip_distance: constant(100.),
        focus_distance: constant(2.),
        far_focus_range: constant(3.),
        near_focus_range: constant(4.),
        dof_amount: constant(0.),
    };
    imported
        .edit(|draft| draft.add_feature(id, surface::Feature::Camera((camera).into())))
        .unwrap();
    let output = io::m3::plan::<m3::StarCraft2>(&imported, &Default::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root_record().unwrap();
    let reference = m3::visit_m3_root!(root, |r| r.cameras);
    let camera = output
        .model
        .container()
        .resolve::<nr::Camera<nr::V2>>(reference.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(camera.reserved_0, 2.);
    assert_eq!(camera.reserved_1, 3.);
    assert_eq!(camera.field_of_view.initial_value, 1.);
}

#[test]
fn inherited_emitters_rebuild_from_independent_graph_definitions() {
    let source = source();
    let id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = source.model.into_container();
    let material = document
        .append(&[nr::StandardMaterial::<nr::V20>::default()])
        .unwrap();
    root.standard_materials = m3::Ref::from_section(1, material.index());
    let map = document
        .append(&[nr::MaterialMap {
            material_type: nr::MaterialKind::Standard,
            material_index: 0,
        }])
        .unwrap();
    root.material_maps = m3::Ref::from_section(1, map.index());
    let ribbon = document
        .append(&[nr::RibbonEmitter::<nr::V9>::default()])
        .unwrap();
    root.ribbon_emitters = m3::Ref::from_section(1, ribbon.index());
    let copy = document
        .append(&[nr::ParticleEmitterCopy {
            emission_rate: nr::AnimationValue {
                initial_value: 12.,
                null_value: 2.,
                ..Default::default()
            },
            squirt_amount: nr::AnimationValue {
                initial_value: 3,
                null_value: 1,
                ..Default::default()
            },
            bone_index: 0,
        }])
        .unwrap();
    root.particle_emitter_copies = m3::Ref::from_section(1, copy.index());
    let indices = document.append(&[nr::U32 { value: 0 }]).unwrap();
    let particle = document
        .append(&[nr::ParticleEmitter::<nr::V24> {
            collision_spawn_index: -1,
            trail_link_index: -1,
            splat_projection_index: -1,
            ribbon_link_index: 0,
            flipbook_column_fraction: f32::INFINITY,
            flipbook_row_fraction: f32::INFINITY,
            copy_indices: m3::Ref::from_section(1, indices.index()),
            ..Default::default()
        }])
        .unwrap();
    root.particle_emitters = m3::Ref::from_section(1, particle.index());
    document.replace(id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Document::from_container(document).unwrap());
    let mut imported = io::m3::import(&source, &Default::default()).unwrap().value;
    let (copy_id, feature) = imported
        .features()
        .find(|(_, feature)| matches!(feature.payload, surface::Feature::ParticleCopy(_)))
        .unwrap();
    let mut payload = feature.payload.clone();
    let surface::Feature::ParticleCopy(copy) = &mut payload else {
        unreachable!()
    };
    copy.emission_rate.initial = 25.;
    let owner = feature.node;
    imported
        .edit(|draft| draft.set_feature(copy_id, payload))
        .unwrap();
    assert_eq!(imported.feature(copy_id).unwrap().node, owner);
    let output = io::m3::plan::<m3::StarCraft2>(&imported, &Default::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let particle = output
        .model
        .container()
        .resolve::<nr::ParticleEmitter<nr::V24>>(root.particle_emitters.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(particle.ribbon_link_index, 0);
    let copy = output
        .model
        .container()
        .resolve::<nr::ParticleEmitterCopy>(root.particle_emitter_copies.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(copy.emission_rate.initial_value, 25.);
    assert_eq!(copy.emission_rate.null_value, 2.);
    assert_eq!(copy.squirt_amount.initial_value, 3);
    let reread = io::m3::import(&output, &Default::default()).unwrap();
    assert_eq!(
        reread
            .value
            .features()
            .filter(|(_, feature)| matches!(feature.payload, surface::Feature::Particle(_)))
            .count(),
        1
    );
    assert_eq!(
        reread
            .value
            .features()
            .filter(|(_, feature)| matches!(feature.payload, surface::Feature::ParticleCopy(_)))
            .count(),
        1
    );
}

#[test]
fn every_particle_record_revision_maps_through_logical_fields() {
    macro_rules! roundtrip {
        ($version:ty $(,$field:ident=$value:expr)*) => {{
            let source=source();
            let id=source.model.root_id();
            let mut root=source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
            let mut document=source.model.into_container();
            let material=document.append(&[nr::StandardMaterial::<nr::V20>::default()]).unwrap();
            root.standard_materials=m3::Ref::from_section(1,material.index());
            let maps=document.append(&[nr::MaterialMap{material_type:nr::MaterialKind::Standard,material_index:0}]).unwrap();
            root.material_maps=m3::Ref::from_section(1,maps.index());
            let ribbon=document.append(&[nr::RibbonEmitter::<nr::V9>::default()]).unwrap();
            root.ribbon_emitters=m3::Ref::from_section(1,ribbon.index());
            let emitter=nr::ParticleEmitter::<$version>{$($field:$value,)*trail_link_index:-1,collision_spawn_index:-1,splat_projection_index:-1,..Default::default()};
            let particles=document.append(&[emitter]).unwrap();
            root.particle_emitters=m3::Ref::from_section(1,particles.index());
            document.replace(id,&[root]).unwrap();
            let source=m3::Bundle::from_model(m3::Document::from_container(document).unwrap());
            let imported=io::m3::import(&source,&Default::default()).unwrap().value;
            let output=io::m3::plan::<m3::StarCraft2>(&imported,&Default::default()).unwrap().execute().unwrap().value;
            let reread=io::m3::import(&output,&Default::default()).unwrap();
            assert_eq!(reread.value.features().filter(|(_,feature)|matches!(feature.payload,surface::Feature::Particle(_))).count(),1);
        }};
    }
    roundtrip!(nr::V10, deprecated_noise_smoothness = 0.375);
    roundtrip!(nr::V11);
    roundtrip!(nr::V12);
    roundtrip!(nr::V13);
    roundtrip!(nr::V14);
    roundtrip!(nr::V15);
    roundtrip!(nr::V16);
    roundtrip!(nr::V17);
    roundtrip!(nr::V18);
    roundtrip!(nr::V19);
    roundtrip!(nr::V20);
    roundtrip!(nr::V21);
    roundtrip!(nr::V22);
    roundtrip!(nr::V23);
    roundtrip!(nr::V24);
}

#[test]
fn static_collision_streams_are_materialized_and_independently_editable() {
    use wc3::model::profiles::Universal;
    use wc3::model::{
        AttributeDomain, AttributeSemantic, GeometryKind, GeometrySchema, M3CollisionGeometry,
        M3Geometry,
    };
    let source = source();
    let id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut document = source.model.into_container();
    let positions = document
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
    let faces = document
        .append(&[
            nr::U16 { value: 0 },
            nr::U16 { value: 1 },
            nr::U16 { value: 2 },
        ])
        .unwrap();
    let normals = document
        .append(&[nr::Vector3 {
            value: [0., 0., 1.],
        }])
        .unwrap();
    root.collision_verts = m3::Ref::from_section(3, positions.index());
    root.collision_faces = m3::Ref::from_section(3, faces.index());
    root.collision_normals = m3::Ref::from_section(1, normals.index());
    root.collision_bounds = nr::Extent {
        minimum: [0.; 3],
        maximum: [1., 1., 0.],
        radius: 1.,
    };
    document.replace(id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Document::from_container(document).unwrap());
    let mut widened = io::m3::import(&source, &Default::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let collision = widened
        .meshes()
        .find(|(_, mesh)| mesh.geometry.kind() == GeometryKind::Collision)
        .unwrap()
        .0;
    widened
        .edit(|draft| {
            let mut mesh = draft.model().mesh(collision).unwrap().clone();
            let primary = mesh.subsets.primary();
            let data = &mut mesh.geometry;
            let index = data
                .attributes
                .iter()
                .position(|attribute| attribute.semantic == AttributeSemantic::Normal)
                .unwrap();
            data.attributes.update(index, |attribute| {
                attribute.domain = AttributeDomain::ConnectivityFace(primary)
            })?;
            draft.set_mesh(collision, mesh)
        })
        .unwrap();
    let mut imported = widened.try_narrow::<M3>().unwrap();
    let (_, surface::System::Collision(system)) = imported.systems().next().unwrap() else {
        panic!("expected collision world")
    };
    let id = system.mesh;
    let mesh = imported.mesh(id).unwrap();
    assert!(mesh.positions().is_owned());
    assert!(mesh.normals().is_none());
    let M3Geometry::Collision(geometry) = &mesh.geometry else {
        panic!("expected collision geometry")
    };
    assert!(geometry.face_normals().unwrap().is_owned());
    let mut edited = mesh.clone();
    edited.geometry = M3Geometry::Collision(
        M3CollisionGeometry::new(
            (vec![[0., 0., 0.], [2., 0., 0.], [0., 1., 0.]]).into_zero_vec(),
            (vec![[0, 1, 2]]).into_zero_vec(),
        )
        .unwrap()
        .with_face_normals((vec![[0., 0., 1.]]).into_zero_vec())
        .unwrap(),
    );
    imported.edit(|draft| draft.set_mesh(id, edited)).unwrap();
    let output = io::m3::plan::<m3::StarCraft2>(&imported, &Default::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    assert_eq!(
        output
            .model
            .container()
            .resolve::<nr::Vector3>(root.collision_verts.cast())
            .unwrap()
            .get(1)
            .unwrap()
            .unwrap()
            .value,
        [2., 0., 0.]
    );
    assert_eq!(root.collision_normals.entries, 1);
}

#[test]
fn joint_relationships_rebuild_from_edited_logical_controls() {
    let mut source = source();
    let root_id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let joint = nr::PhysicsJoint {
        joint_type: nr::JointKind::Revolute,
        bone_index1: 0,
        bone_index2: 0,
        enable_limits: 0x8000_0001,
        enable_friction: 0x4000_0001,
        friction: 0.125,
        collision_control: -17,
        ..Default::default()
    };
    let mut document = source.model.into_container();
    let joints = document.append(&[joint]).unwrap();
    root.physics_joints = m3::Ref::from_section(1, joints.index());
    document.replace(root_id, &[root]).unwrap();
    source.model = m3::Document::from_container(document).unwrap();
    let mut asset = io::m3::import(&source, &Default::default()).unwrap().value;
    let (id, system) = asset
        .systems()
        .find(|(_, system)| matches!(system, surface::System::Physics(_)))
        .unwrap();
    let mut system = system.clone();
    let surface::System::Physics(physics) = &mut system else {
        unreachable!()
    };
    physics.joints[0].friction = 0.75;
    asset
        .edit(|builder| builder.set_system(id, system))
        .unwrap();
    let output = io::m3::plan::<m3::StarCraft2>(&asset, &Default::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let joints = output
        .model
        .container()
        .resolve::<nr::PhysicsJoint>(root.physics_joints.cast())
        .unwrap();
    let joint = joints.get(0).unwrap().unwrap();
    assert_eq!(joint.friction, 0.75);
    assert_eq!(joint.enable_limits, 0x8000_0001);
    assert_eq!(joint.enable_friction, 0x4000_0001);
    assert_eq!(joint.collision_control, -17);
}

#[test]
fn body_mass_and_dynamic_defaults_are_logical_authoring_values() {
    let mut source = source();
    let root_id = source.model.root_id();
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let body = nr::RigidBody::<nr::V4> {
        simulation_type: 0x1234,
        mass: 12.5,
        gravity_factor: 0.25,
        dynamic_state: nr::AnimationValue {
            animation_id: 0x1234_5678,
            initial_value: u32::MAX,
            null_value: 16_777_217,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut document = source.model.into_container();
    let bodies = document.append(&[body]).unwrap();
    root.rigid_bodies = m3::Ref::from_section(1, bodies.index());
    document.replace(root_id, &[root]).unwrap();
    source.model = m3::Document::from_container(document).unwrap();
    let mut asset = io::m3::import(&source, &Default::default()).unwrap().value;
    let (id, system) = asset
        .systems()
        .find(|(_, system)| matches!(system, surface::System::Physics(_)))
        .unwrap();
    let mut system = system.clone();
    let surface::System::Physics(physics) = &mut system else {
        unreachable!()
    };
    let parameters = physics.bodies[0].parameters.as_mut().unwrap();
    assert_eq!(parameters.mass, 12.5);
    parameters.mass = 25.;
    asset
        .edit(|builder| builder.set_system(id, system))
        .unwrap();
    let output = io::m3::plan::<m3::StarCraft2>(&asset, &Default::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let bodies = output
        .model
        .container()
        .resolve::<nr::RigidBody<nr::V4>>(root.rigid_bodies.cast())
        .unwrap();
    let body = bodies.get(0).unwrap().unwrap();
    assert_eq!(body.mass, 25.);
    assert_eq!(body.gravity_factor, 0.25);
    assert_eq!(body.simulation_type, 0x1234);
    assert_eq!(body.dynamic_state.initial_value, u32::MAX);
    assert_eq!(body.dynamic_state.null_value, 16_777_217);
}
