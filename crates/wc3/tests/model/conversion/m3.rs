use std::borrow::Cow;
use wc3::model::{
    conversion::{self, ApproximationPolicy, Asset, ExportOptions, ImportOptions},
    m3::{self, records as nr},
    wem::{self, semantic as sem},
};
fn asset() -> Asset {
    let node = sem::Node {
        name: Cow::Borrowed("Root"),
        bone: true,
        ..Default::default()
    };
    let mesh = sem::Mesh {
        name: Cow::Borrowed("Triangle"),
        positions: Cow::Owned(vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]),
        normals: Cow::Owned(vec![[0., 0., 1.]; 3]),
        uv_sets: vec![Cow::Owned(vec![[0., 0.], [1., 0.], [0., 1.]])],
        triangles: Cow::Owned(vec![[0, 1, 2]]),
        influences: vec![
            vec![sem::Influence {
                node: 0,
                weight: 1.
            }];
            3
        ],
        ..Default::default()
    };
    let clip = sem::Clip {
        name: Cow::Borrowed("Move"),
        duration: 1.,
        tracks: vec![sem::Track {
            layer: Default::default(),
            target: sem::Target::Translation(0),
            interpolation: sem::Interpolation::Linear,
            global_duration: None,
            keys: vec![
                sem::Key {
                    time: 0.,
                    value: vec![0., 0., 0.],
                    incoming: None,
                    outgoing: None,
                },
                sem::Key {
                    time: 1.,
                    value: vec![1., 2., 3.],
                    incoming: None,
                    outgoing: None,
                },
            ],
        }],
        ..Default::default()
    };
    let model = sem::Model {
        name: Cow::Borrowed("Triangle"),
        nodes: vec![node],
        meshes: vec![mesh],
        materials: vec![sem::Material {
            name: Cow::Borrowed("Solid"),
            ..Default::default()
        }],
        clips: vec![clip],
        profile: wem::records::ProfileId::SC2,
        ..Default::default()
    };
    Asset::new(wem::semantic::build(&model).unwrap())
}
fn approximate() -> ExportOptions {
    ExportOptions {
        approximation: ApproximationPolicy::Allow { tolerance: 0.004 },
        ..Default::default()
    }
}
#[test]
fn geometry_skin_and_animation_survive_native_round_trip() {
    let native = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap();
    assert!(!native.report.issues.is_empty());
    let encoded = native.value.write().unwrap();
    let model = m3::Model::<m3::Md34>::read(&encoded.model).unwrap();
    let root = model.root_record().unwrap();
    assert_eq!(root.revision(), 29);
    let bone = model
        .document()
        .resolve::<nr::Bone<nr::V1>>(root.bones().cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_ne!(bone.position.animation_id, 0);
    let imported =
        conversion::m3::import(&m3::Bundle::from_model(model), &ImportOptions::default()).unwrap();
    let semantic = imported.value.model(&ExportOptions::default()).unwrap();
    assert_eq!(semantic.meshes.len(), 1);
    assert_eq!(&*semantic.meshes[0].triangles, &[[0, 1, 2]]);
    assert_eq!(semantic.meshes[0].positions[1], [1., 0., 0.]);
    assert_eq!(semantic.meshes[0].influences[2][0].weight, 1.);
    assert_eq!(semantic.nodes[0].name, "Root");
    assert_eq!(semantic.clips.len(), 1);
    assert_eq!(semantic.clips[0].tracks[0].keys[1].value, [1., 2., 3.]);
}
#[test]
fn quantization_requires_an_explicit_policy() {
    let error =
        conversion::m3::plan::<m3::StarCraft2>(&asset(), &ExportOptions::default()).unwrap_err();
    assert!(error.path.contains("normal"));
}
#[test]
fn profiles_select_root_and_envelope_independently() {
    let asset = asset();
    let beta = conversion::m3::plan::<m3::Beta>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap();
    assert_eq!(beta.value.model.root_record().unwrap().revision(), 23);
    assert_eq!(&beta.value.write().unwrap().model[..4], b"33DM");
    let heroes = conversion::m3::plan::<m3::HeroesOfTheStorm>(&asset, &approximate())
        .unwrap()
        .execute()
        .unwrap();
    assert_eq!(heroes.value.model.root_record().unwrap().revision(), 30);
}
#[test]
fn nonexistent_node_tracks_are_rejected() {
    let source = asset();
    let mut model = source.model(&ExportOptions::default()).unwrap().clone();
    model.clips[0].tracks[0].target = sem::Target::Translation(3);
    let error = wem::semantic::build(&model).unwrap_err();
    assert!(error.to_string().contains("semantic model"));
}

#[test]
fn native_standard_controls_remain_active_after_geometry_edit() {
    let source = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = match source.model.root_record().unwrap() {
        m3::RootRecord::V29(r) => r,
        _ => unreachable!(),
    };
    let mut document = source.model.into_document();
    let mut material = document
        .resolve::<nr::StandardMaterial<nr::V20>>(root.standard_materials.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    material.priority = -7;
    material.specular_exponent = 25.;
    material.alpha_test_threshold = 97;
    material.hdr_environment_constant = 2.5;
    let section = document.section_id(root.standard_materials.index).unwrap();
    document.replace(section, &[material]).unwrap();
    let imported = conversion::m3::import(
        &m3::Bundle::from_model(m3::Model::from_document(document).unwrap()),
        &ImportOptions::default(),
    )
    .unwrap();
    let semantic = imported.value.model(&ExportOptions::default()).unwrap();
    assert!(semantic.native.is_some());
    // Edit only shared geometry while retaining its native profile graph.
    let mut raw = imported.value.document.clone().into_raw();
    let root = raw
        .resolve::<wem::records::DocumentRecord<wem::records::V1>>(wem::Ref::new(
            raw.root_reference().entries,
            raw.root_reference().index,
            raw.root_reference().flags,
        ))
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let model = raw
        .resolve::<wem::records::ModelRecord<wem::records::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mesh = raw
        .resolve::<wem::records::Mesh<wem::records::V2>>(model.meshes.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let attribute = raw
        .resolve::<wem::records::AttrLayer>(mesh.attributes.layers.cast())
        .unwrap()
        .iter()
        .map(Result::unwrap)
        .find(|a| raw.string(a.name).unwrap() == Some("position"))
        .unwrap();
    let section = raw.section_id(attribute.data.index).unwrap();
    let mut bytes = raw.section(section).unwrap().bytes().to_vec();
    bytes[..4].copy_from_slice(&3f32.to_le_bytes());
    raw.replace_bytes(section, bytes).unwrap();
    let edited = Asset::new(wem::Document::from_raw(raw).unwrap());
    let exported = conversion::m3::plan::<m3::StarCraft2>(&edited, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = match exported.model.root_record().unwrap() {
        m3::RootRecord::V29(r) => r,
        _ => unreachable!(),
    };
    let actual = exported
        .model
        .document()
        .resolve::<nr::StandardMaterial<nr::V20>>(root.standard_materials.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(actual.priority, -7);
    assert_eq!(actual.specular_exponent, 25.);
    assert_eq!(actual.alpha_test_threshold, 97);
    assert_eq!(actual.hdr_environment_constant, 2.5);
}
#[test]
fn particle_copies_ribbons_and_spline_controls_are_active() {
    let source = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let mut root = source.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut doc = source.model.into_document();
    let root_id = doc.section_id(doc.root_reference().index).unwrap();
    let point = nr::SplineRibbon {
        bone_index: 0,
        emission_offset: [1., 2., 3.],
        velocity: nr::AnimationValue {
            initial_value: 7.,
            null_value: 2.,
            ..Default::default()
        },
        yaw_type: 3,
        emission_vector_norm_factor: 0.25,
        ..Default::default()
    };
    let spline = doc.append(&[point]).unwrap();
    root.ribbon_emitters = doc
        .append(&[nr::RibbonEmitter::<nr::V9> {
            bone_index: 0,
            spline_ribbons: m3::Ref::new(1, spline.index(), 0),
            drag: 2.5,
            ..Default::default()
        }])
        .map(|id| m3::Ref::new(1, id.index(), 0))
        .unwrap();
    let copies = doc
        .append(&[nr::ParticleEmitterCopy {
            bone_index: 0,
            emission_rate: nr::AnimationValue {
                initial_value: 5.,
                ..Default::default()
            },
            squirt_amount: nr::AnimationValue {
                initial_value: 3,
                ..Default::default()
            },
        }])
        .unwrap();
    root.particle_emitter_copies = m3::Ref::new(1, copies.index(), 0);
    let copy_ids = doc.append(&[nr::U32 { value: 0 }]).unwrap();
    root.particle_emitters = doc
        .append(&[nr::ParticleEmitter::<nr::V24> {
            bone_index: 0,
            emission_rate: nr::AnimationValue {
                initial_value: 1.,
                ..Default::default()
            },
            copy_indices: m3::Ref::new(1, copy_ids.index(), 0),
            collision_spawn_index: -1,
            trail_link_index: -1,
            ribbon_link_index: 0,
            drag: 4.5,
            ..Default::default()
        }])
        .map(|id| m3::Ref::new(1, id.index(), 0))
        .unwrap();
    doc.replace(root_id, &[root]).unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(doc).unwrap());
    let imported = conversion::m3::import(&source, &ImportOptions::default()).unwrap();
    let semantic = imported.value.model(&ExportOptions::default()).unwrap();
    assert_eq!(semantic.nodes.len(), 4);
    let output = conversion::m3::plan::<m3::StarCraft2>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    assert_eq!(root.particle_emitters.entries, 1);
    assert_eq!(root.particle_emitter_copies.entries, 1);
    assert_eq!(root.ribbon_emitters.entries, 1);
    assert_eq!(root.bones.entries, 1);
    let doc = output.model.document();
    let row = doc
        .resolve::<nr::ParticleEmitter<nr::V24>>(root.particle_emitters.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(row.drag, 4.5);
    assert_eq!(row.ribbon_link_index, 0);
    let copy = doc
        .resolve::<nr::ParticleEmitterCopy>(root.particle_emitter_copies.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(copy.emission_rate.initial_value, 5.);
    assert_eq!(copy.squirt_amount.initial_value, 3);
    let row = doc
        .resolve::<nr::RibbonEmitter<nr::V9>>(root.ribbon_emitters.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let point = doc
        .resolve::<nr::SplineRibbon>(row.spline_ribbons.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(point.emission_offset, [1., 2., 3.]);
    assert_eq!(point.velocity.initial_value, 7.);
    assert_eq!(point.velocity.null_value, 2.);
    assert_eq!(point.yaw_type, 3);
}
#[test]
fn collision_and_selection_shapes_survive_without_becoming_dynamic_bodies() {
    let base = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let mut root = base.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut doc = base.model.into_document();
    let root_id = doc.section_id(doc.root_reference().index).unwrap();
    let points = doc
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
    let faces = doc
        .append(&[
            nr::U16 { value: 0 },
            nr::U16 { value: 1 },
            nr::U16 { value: 2 },
        ])
        .unwrap();
    let normals = doc
        .append(&[nr::Vector3 {
            value: [0., 0., 1.],
        }])
        .unwrap();
    root.collision_verts = m3::Ref::new(3, points.index(), 0);
    root.collision_faces = m3::Ref::new(3, faces.index(), 0);
    root.collision_normals = m3::Ref::new(1, normals.index(), 0);
    root.collision_bounds = nr::Extent {
        minimum: [0.; 3],
        maximum: [1., 1., 0.],
        radius: 1.,
    };
    root.tight_hit_test_object = nr::HitTestShape {
        shape_type: 1,
        bone_index: 0,
        padding: 9,
        transform: [
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [2., 3., 4., 1.],
        ],
        size_x: 2.,
        size_y: 3.,
        size_z: 4.,
        ..Default::default()
    };
    doc.replace(root_id, &[root]).unwrap();
    let imported = conversion::m3::import(
        &m3::Bundle::from_model(m3::Model::from_document(doc).unwrap()),
        &ImportOptions::default(),
    )
    .unwrap();
    let output = conversion::m3::plan::<m3::StarCraft2>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    assert_eq!(root.rigid_bodies.entries, 0);
    assert_eq!(root.collision_faces.entries, 3);
    assert_eq!(root.collision_verts.entries, 3);
    assert_eq!(root.collision_normals.entries, 1);
    assert_eq!(root.collision_bounds.radius, 1.);
    assert_eq!(root.tight_hit_test_object.padding, 9);
    assert_eq!(root.tight_hit_test_object.size_x, 2.);
    assert_eq!(root.tight_hit_test_object.transform[3], [2., 3., 4., 1.]);
}

#[test]
fn full_width_unsigned_animation_keys_remain_typed_and_exact() {
    use wc3::model::wem::records as wr;
    let base = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = base.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut doc = base.model.into_document();
    let mut bone = doc
        .resolve::<nr::Bone<nr::V1>>(root.bones.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    bone.batching.animation_id = 555;
    bone.batching.flags = 1;
    doc.replace(doc.section_id(root.bones.index).unwrap(), &[bone])
        .unwrap();
    let mut controller = doc
        .resolve::<nr::SubTrackContainer<nr::V4>>(root.sub_track_collections.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut ids = doc
        .resolve::<nr::U32>(controller.anim_ids.cast())
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut selectors = doc
        .resolve::<nr::U32>(controller.anim_refs.cast())
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    ids.push(nr::U32 { value: 555 });
    selectors.push(nr::U32 { value: 10 << 16 });
    controller.anim_ids =
        m3::Ref::<nr::U32>::from_section(ids.len() as u32, doc.append(&ids).unwrap().index())
            .cast();
    controller.anim_refs = m3::Ref::<nr::U32>::from_section(
        selectors.len() as u32,
        doc.append(&selectors).unwrap().index(),
    )
    .cast();
    let timestamps = m3::Ref::<nr::I32>::from_section(
        2,
        doc.append(&[nr::I32 { value: 0 }, nr::I32 { value: 1000 }])
            .unwrap()
            .index(),
    )
    .cast();
    let keys = m3::Ref::<nr::U32>::from_section(
        2,
        doc.append(&[nr::U32 { value: 0xfffffff1 }, nr::U32 { value: u32::MAX }])
            .unwrap()
            .index(),
    )
    .cast();
    let block = nr::AnimationBlockU32 {
        timestamps,
        keys,
        flags: 9,
        end_frame: 1000,
    };
    controller.sdu3 =
        m3::Ref::<nr::AnimationBlockU32>::from_section(1, doc.append(&[block]).unwrap().index())
            .cast();
    doc.replace(
        doc.section_id(root.sub_track_collections.index).unwrap(),
        &[controller],
    )
    .unwrap();
    let source = m3::Bundle::from_model(m3::Model::from_document(doc).unwrap());
    let imported = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let root = imported
        .document
        .root::<wr::DocumentRecord<wr::V1>>()
        .unwrap();
    let model = imported
        .document
        .raw()
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let channel = imported
        .document
        .raw()
        .resolve::<wr::AnimChannel>(model.anim_channels.channels.cast())
        .unwrap()
        .iter()
        .find_map(|channel| {
            let channel = channel.unwrap();
            (channel.id == 555).then_some(channel)
        })
        .unwrap();
    assert_eq!(channel.value_type, wr::AttrType::U32);
    let output = conversion::m3::plan::<m3::StarCraft2>(&imported, &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let controller = output
        .model
        .document()
        .resolve::<nr::SubTrackContainer<nr::V4>>(root.sub_track_collections.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let block = output
        .model
        .document()
        .resolve::<nr::AnimationBlockU32>(controller.sdu3.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let keys = output
        .model
        .document()
        .resolve::<nr::U32>(block.keys.cast())
        .unwrap();
    assert_eq!(keys.get(0).unwrap().unwrap().value, 0xfffffff1);
    assert_eq!(keys.get(1).unwrap().unwrap().value, u32::MAX);
    assert_eq!(block.flags, 9);
    let bone = output
        .model
        .document()
        .resolve::<nr::Bone<nr::V1>>(root.bones.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(bone.batching.animation_id, 555);
}
#[test]
fn solver_controls_are_named_editable_fields() {
    let base = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let mut root = base.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut doc = base.model.into_document();
    let root_id = doc.section_id(doc.root_reference().index).unwrap();
    let dep = doc.append(&[nr::U16 { value: 0 }]).unwrap();
    let ik = doc
        .append(&[nr::IKJoint {
            dependents: m3::Ref::new(1, dep.index(), 0),
            bone_index1: 0,
            bone_index2: 0,
            raycast_up: 1.5,
            raycast_down: -2.,
            max_speed: 3.,
            goal_threshold: 0.25,
        }])
        .unwrap();
    root.ik_joints = m3::Ref::new(1, ik.index(), 0);
    let bb = doc
        .append(&[nr::BillboardBehavior {
            bone_index: 0,
            billboard_type: 6,
            camera_look_at: 1,
            up: [0., 0., 0., 1.],
            forward: [1., 0., 0., 0.],
            ..Default::default()
        }])
        .unwrap();
    root.billboard_behaviors = m3::Ref::new(1, bb.index(), 0);
    doc.replace(root_id, &[root]).unwrap();
    let imported = conversion::m3::import(
        &m3::Bundle::from_model(m3::Model::from_document(doc).unwrap()),
        &ImportOptions::default(),
    )
    .unwrap();
    let mut raw = imported.value.document.into_raw();
    let root = raw
        .resolve::<wem::records::DocumentRecord<wem::records::V1>>(wem::Ref::from_section(
            raw.root_reference().entries,
            raw.root_reference().index,
        ))
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let model = raw
        .resolve::<wem::records::ModelRecord<wem::records::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let set = raw
        .resolve::<wem::records::ProfileMaterialSet>(model.profile_sets.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut entries = raw
        .resolve::<wem::records::NativeEntry<wem::records::V2>>(set.native.entries.cast())
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let entry = entries
        .iter_mut()
        .find(|e| raw.string(e.name).unwrap() == Some("m3.controls.ik_joints[0].max_speed"))
        .unwrap();
    entry.value = i64::from(7f32.to_bits());
    let id = raw.section_id(set.native.entries.index).unwrap();
    raw.replace(id, &entries).unwrap();
    let edited = Asset::new(wem::Document::from_raw(raw).unwrap());
    let output = conversion::m3::plan::<m3::StarCraft2>(&edited, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let doc = output.model.document();
    let ik = doc
        .resolve::<nr::IKJoint>(root.ik_joints.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(ik.max_speed, 7.);
    assert_eq!(ik.raycast_down, -2.);
    assert_eq!(ik.goal_threshold, 0.25);
    let bb = doc
        .resolve::<nr::BillboardBehavior>(root.billboard_behaviors.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(bb.billboard_type, 6);
    assert_eq!(bb.camera_look_at, 1);
    assert_eq!(bb.forward, [1., 0., 0., 0.]);
}
#[test]
fn every_particle_and_ribbon_record_revision_rebuilds_from_wem_fields() {
    macro_rules! particle {
        ($v:ty) => {{
            let base = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
                .unwrap()
                .execute()
                .unwrap()
                .value;
            let mut root = base.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
            let mut doc = base.model.into_document();
            let id = doc.section_id(doc.root_reference().index).unwrap();
            let row = nr::ParticleEmitter::<$v> {
                bone_index: 0,
                collision_spawn_index: -1,
                trail_link_index: -1,
                ribbon_link_index: disabled_link(),
                drag: 0.75,
                ..Default::default()
            };
            let section = doc.append(&[row]).unwrap();
            root.particle_emitters = m3::Ref::new(1, section.index(), 0);
            doc.replace(id, &[root]).unwrap();
            let imported = conversion::m3::import(
                &m3::Bundle::from_model(m3::Model::from_document(doc).unwrap()),
                &ImportOptions::default(),
            )
            .unwrap();
            let output =
                conversion::m3::plan::<m3::StarCraft2>(&imported.value, &ExportOptions::default())
                    .unwrap()
                    .execute()
                    .unwrap()
                    .value;
            let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
            let doc = output.model.document();
            let row = doc
                .resolve::<nr::ParticleEmitter<$v>>(root.particle_emitters.cast())
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap();
            assert_eq!(row.drag, 0.75);
        }};
    }
    particle!(nr::V10);
    particle!(nr::V11);
    particle!(nr::V12);
    particle!(nr::V13);
    particle!(nr::V14);
    particle!(nr::V15);
    particle!(nr::V16);
    particle!(nr::V17);
    particle!(nr::V18);
    particle!(nr::V19);
    particle!(nr::V20);
    particle!(nr::V21);
    particle!(nr::V22);
    particle!(nr::V23);
    particle!(nr::V24);
    macro_rules! ribbon {
        ($v:ty) => {{
            let base = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
                .unwrap()
                .execute()
                .unwrap()
                .value;
            let mut root = base.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
            let mut doc = base.model.into_document();
            let id = doc.section_id(doc.root_reference().index).unwrap();
            let row = nr::RibbonEmitter::<$v> {
                bone_index: 0,
                drag: 0.75,
                ..Default::default()
            };
            let section = doc.append(&[row]).unwrap();
            root.ribbon_emitters = m3::Ref::new(1, section.index(), 0);
            doc.replace(id, &[root]).unwrap();
            let imported = conversion::m3::import(
                &m3::Bundle::from_model(m3::Model::from_document(doc).unwrap()),
                &ImportOptions::default(),
            )
            .unwrap();
            let output =
                conversion::m3::plan::<m3::StarCraft2>(&imported.value, &ExportOptions::default())
                    .unwrap()
                    .execute()
                    .unwrap()
                    .value;
            let root = output.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
            let row = output
                .model
                .document()
                .resolve::<nr::RibbonEmitter<$v>>(root.ribbon_emitters.cast())
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap();
            assert_eq!(row.drag, 0.75);
        }};
    }
    ribbon!(nr::V4);
    ribbon!(nr::V5);
    ribbon!(nr::V6);
    ribbon!(nr::V7);
    ribbon!(nr::V8);
    ribbon!(nr::V9);
}

trait DisabledLink {
    fn disabled() -> Self;
}
impl DisabledLink for () {
    fn disabled() -> Self {}
}
impl DisabledLink for i32 {
    fn disabled() -> Self {
        -1
    }
}
fn disabled_link<T: DisabledLink>() -> T {
    T::disabled()
}

#[test]
fn dormant_standard_shaders_remain_editable_native_materials() {
    let source = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let mut root = match source.model.root_record().unwrap() {
        m3::RootRecord::V29(r) => r,
        _ => unreachable!(),
    };
    let mut document = source.model.into_document();
    let first = document
        .resolve::<nr::StandardMaterial<nr::V20>>(root.standard_materials.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut dormant = first;
    dormant.specular_exponent = 37.;
    dormant.flags = 0x20;
    dormant.blend_mode = 3;
    let id = document.section_id(root.standard_materials.index).unwrap();
    document.replace(id, &[first, dormant]).unwrap();
    root.standard_materials.entries = 2;
    let root_id = document
        .section_id(document.root_reference().index)
        .unwrap();
    document.replace(root_id, &[root]).unwrap();
    let imported = conversion::m3::import(
        &m3::Bundle::from_model(m3::Model::from_document(document).unwrap()),
        &ImportOptions::default(),
    )
    .unwrap()
    .value;
    assert_eq!(
        imported
            .model(&ExportOptions::default())
            .unwrap()
            .materials
            .len(),
        2
    );
    let exported = conversion::m3::plan::<m3::StarCraft2>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = match exported.model.root_record().unwrap() {
        m3::RootRecord::V29(r) => r,
        _ => unreachable!(),
    };
    assert_eq!(root.standard_materials.entries, 2);
    let dormant = exported
        .model
        .document()
        .resolve::<nr::StandardMaterial<nr::V20>>(root.standard_materials.cast())
        .unwrap()
        .get(1)
        .unwrap()
        .unwrap();
    assert_eq!(dormant.specular_exponent, 37.);
    assert_eq!(dormant.flags, 0x20);
    assert_eq!(dormant.blend_mode, 3);
}

#[test]
fn dormant_hair_shaders_require_an_explicit_loss_policy() {
    let source = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let mut root = match source.model.root_record().unwrap() {
        m3::RootRecord::V29(r) => r,
        _ => unreachable!(),
    };
    let mut document = source.model.into_document();
    let hair = document.append(&[nr::HairMaterial::default()]).unwrap();
    root.hair_materials = m3::Ref::from_section(1, hair.index());
    let root_id = document
        .section_id(document.root_reference().index)
        .unwrap();
    document.replace(root_id, &[root]).unwrap();
    let bundle = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let error = conversion::m3::import(&bundle, &ImportOptions::default()).unwrap_err();
    assert!(error.to_string().contains("hair_materials"));
    let imported = conversion::m3::import(
        &bundle,
        &ImportOptions {
            loss_policy: conversion::LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "m3.hair_materials"));
}

#[test]
fn referenced_hair_shader_can_be_dropped_into_a_generic_material() {
    let source = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let mut root = match source.model.root_record().unwrap() {
        m3::RootRecord::V29(r) => r,
        _ => unreachable!(),
    };
    let mut document = source.model.into_document();
    let hair = document.append(&[nr::HairMaterial::default()]).unwrap();
    root.hair_materials = m3::Ref::from_section(1, hair.index());
    let maps = document.section_id(root.material_maps.index).unwrap();
    document
        .replace(
            maps,
            &[nr::MaterialMap {
                material_type: 8,
                material_index: 0,
            }],
        )
        .unwrap();
    let root_id = document
        .section_id(document.root_reference().index)
        .unwrap();
    document.replace(root_id, &[root]).unwrap();
    let bundle = m3::Bundle::from_model(m3::Model::from_document(document).unwrap());
    let imported = conversion::m3::import(
        &bundle,
        &ImportOptions {
            loss_policy: conversion::LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    let model = imported.value.model(&ExportOptions::default()).unwrap();
    assert_eq!(model.meshes[0].material_slot, 0);
    assert_eq!(model.materials[0].flags, 0);
    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "m3.materials[0]"));
    conversion::m3::plan::<m3::StarCraft2>(&imported.value, &ExportOptions::default()).unwrap();
}

#[test]
fn companion_static_payloads_require_an_explicit_loss_policy() {
    let mut source = conversion::m3::plan::<m3::StarCraft2>(&asset(), &approximate())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    source
        .insert_animation(0x1234, source.model.clone().into())
        .unwrap();
    let error = conversion::m3::import(&source, &ImportOptions::default()).unwrap_err();
    assert!(error.to_string().contains("animations[00001234].vertices"));
    let imported = conversion::m3::import(
        &source,
        &ImportOptions {
            loss_policy: conversion::LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        imported
            .value
            .model(&ExportOptions::default())
            .unwrap()
            .meshes
            .len(),
        1
    );
    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "m3.animations[00001234].vertices"));
}

#[path = "m3_geometry_controls.rs"]
mod geometry_controls;
