use super::*;
use crate::{compile_model, BakeSpace, Playback, PoseBaker, PoseOptions};
use bevy::math::Mat4;
use bevy::math::Vec3;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use wc3::model::common::PackedArray;
use wc3::model::materials::mdlx::{MaterialInputs, MaterialStep as MdlxStep, Shader};
use wc3::model::profiles::mdlx::Material as MdlxMaterial;
use wc3::model::ModelMetadata;
use wc3::model::{
    Clip, Curve, Draw, DynamicTrack, ExplicitBinding, GeometryData, GeometryInfluences, Influence,
    Material, MaterialSurface, Mesh as ModelMesh, MeshInstance, Node as ModelNode, Quat, Skin,
    Time, TimeSource, Track, UniversalBinding,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{ModelMount, MountTarget, Transform as ModelTransform};

fn model(rest: f32) -> (Model<'static>, NodeId, ClipId) {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let joint = builder
        .add_node(
            ModelNode::named("joint", &mut slot_values)
                .with_transform(
                    ModelTransform {
                        translation: Vec3::from_array([rest, 0., 0.]),
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    &mut slot_values,
                )
                .unwrap(),
        )
        .unwrap();
    let skin = builder
        .add_skin(Skin::new(
            vec![joint],
            UniversalBinding::Explicit(
                ExplicitBinding::new(
                    ([Mat4::from_translation(Vec3::new(-rest, 0., 0.)).to_cols_array_2d()])
                        .into_iter()
                        .map(|v| Mat4::from_cols_array_2d(&v))
                        .collect::<PackedArray<_>>(),
                )
                .unwrap(),
            ),
        ))
        .unwrap();
    let mut geometry = GeometryData::triangles(
        ([[rest, 0., 0.], [rest + 0.5, 0., 0.], [rest, 0.5, 0.]])
            .into_iter()
            .map(Vec3::from_array)
            .collect::<PackedArray<_>>(),
        [[0, 1, 2]],
    )
    .unwrap();
    geometry.influences = GeometryInfluences::Four(
        vec![
            [
                Influence {
                    joint: 0,
                    weight: 1.
                },
                Influence {
                    joint: 0,
                    weight: 0.
                },
                Influence {
                    joint: 0,
                    weight: 0.
                },
                Influence {
                    joint: 0,
                    weight: 0.
                }
            ];
            3
        ]
        .into_iter()
        .collect(),
    );
    let mut mesh = ModelMesh::new(geometry);
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let material = builder
        .add_material(Material {
            name: "plain".into(),
            surface: MaterialSurface::from(MdlxMaterial {
                declaration: Default::default(),
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
    let mut instance = MeshInstance::new(joint, mesh);
    instance.skin = Some(skin);
    instance.draws.push(Draw::new(
        subset,
        SlotConstruct::construct(&mut slot_values),
    ));
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    let clip = builder
        .add_clip(Clip::named("turn").with_duration(Time::seconds(1.).unwrap()))
        .unwrap();
    builder
        .add_track(
            clip,
            DynamicTrack::Rotation(Track {
                target: builder.model().node(joint).unwrap().transform.rotation,
                curve: Curve::linear(
                    [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                    [
                        Quat::IDENTITY,
                        Quat::from_array(
                            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array(),
                        ),
                    ],
                )
                .unwrap(),
                clock: TimeSource::Animation,
            }),
        )
        .unwrap();
    (
        {
            builder.import_values(&slot_values).unwrap();
            builder.finish()
        }
        .unwrap(),
        joint,
        clip,
    )
}
fn position(model: &CompiledModel, clip: ClipId, time: f64, part: usize) -> Vec3 {
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = model.prepare(&mut meshes, &mut binds);
    let mut playback = Playback::for_clip(clip);
    playback.clip_time = time;
    let baked = PoseBaker::default()
        .bake(
            &prepared,
            &meshes,
            &binds,
            PoseOptions {
                playback: Some(&playback),
                space: BakeSpace::Authored,
                ..Default::default()
            },
        )
        .unwrap();
    let VertexAttributeValues::Float32x3(values) = baked.parts[part]
        .mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
    else {
        panic!("positions")
    };
    Vec3::from_array(values[0])
}
#[test]
fn skinned_equipment_reuses_host_joints_and_preserves_distinct_bind_spaces() {
    let (body, joint, clip) = model(1.);
    let (armor, armor_joint, _) = model(3.);
    let body = compile_model(&body, |_| None).unwrap();
    let armor = compile_model(&armor, |_| None).unwrap();
    let composed = body
        .compose([ModelPart::new(
            &armor,
            Skinned {
                joints: JointMap::new([(armor_joint, joint)]).unwrap(),
                transform: Mat4::IDENTITY,
            },
        )])
        .unwrap();
    // Only the host joint is animated. Equipment adds a coordinate frame,
    // while its skin palette addresses the existing host entity.
    assert_eq!(composed.skins[1].joints, body.skins[0].joints);
    assert_eq!(
        composed
            .nodes
            .iter()
            .filter(|node| node.name == "joint")
            .count(),
        1
    );
    assert!(position(&composed, clip, 0., 1).abs_diff_eq(Vec3::new(3., 0., 0.), 1e-5));
    assert!(position(&composed, clip, 0.5, 1)
        .abs_diff_eq(Vec3::new(1. + 2f32.sqrt(), 2f32.sqrt(), 0.), 1e-5));
}
#[test]
fn binding_failures_are_atomic_and_never_guess_joint_ordinals() {
    let (body, _, _) = model(1.);
    let (armor, armor_joint, _) = model(3.);
    let body = compile_model(&body, |_| None).unwrap();
    let armor = compile_model(&armor, |_| None).unwrap();
    assert!(body
        .compose([ModelPart::new(
            &armor,
            Skinned {
                joints: JointMap::new([(armor_joint, armor_joint)]).unwrap(),
                transform: Mat4::IDENTITY
            }
        )])
        .is_err());
    assert!(body
        .compose([ModelPart::new(
            &armor,
            Skinned {
                joints: JointMap::default(),
                transform: Mat4::IDENTITY
            }
        )])
        .is_err());
    assert_eq!(body.draws.len(), 1);
    assert_eq!(body.nodes.len(), 1);
}
#[test]
fn rigid_equipment_uses_mount_frame_and_can_sync_its_clip() {
    let mut slot_values = SlotValues::default();
    let (body, joint, clip) = model(1.);
    let (armor, _, armor_clip) = model(3.);
    let body = compile_model(&body, |_| None).unwrap();
    let armor = compile_model(&armor, |_| None).unwrap();
    let mut part = ModelPart::new(
        &armor,
        Mounted(ModelMount {
            target: MountTarget::Node(joint),
            transform: ModelTransform {
                translation: Vec3::Y,
                ..SlotConstruct::construct(&mut slot_values)
            },
            ..ModelMount::new(MountTarget::Node(joint))
        }),
    );
    part.clips.insert(clip, armor_clip);
    let composed = body.compose([part]).unwrap();
    assert!(position(&composed, clip, 0., 1).abs_diff_eq(Vec3::new(4., 1., 0.), 1e-5));
    assert!(position(&composed, clip, 0.5, 1).is_finite());
}

#[test]
fn mapped_attachment_anchor_retains_source_pivot_and_rest_space() {
    let mut slot_values = SlotValues::default();
    use crate::pose::{local_matrix, PoseContext};
    use wc3::model::{AnimationSample, Attachment, NodeFeature, Socket, SocketIdentity};
    let (host, joint, clip) = model(1.);
    let (mut source, source_joint, _) = model(3.);
    let mut source_node = source.node(source_joint).unwrap().clone();
    source_node.transform.pivot = Vec3::from_array([0., 0., 2.]);
    let feature = source
        .edit(|builder| {
            let edit_result = (|| {
                builder.set_node(source_joint, source_node)?;
                builder.add_feature(
                    source_joint,
                    NodeFeature::Attachment(Attachment::Socket(Socket {
                        identity: SocketIdentity::Role(11),
                        visibility: slot_values.add(1.),
                        control: None,
                    })),
                )
            })();
            builder.import_values(&slot_values)?;
            edit_result
        })
        .unwrap();
    let host = compile_model(&host, |_| None).unwrap();
    let source = compile_model(&source, |_| None).unwrap();
    let composed = host
        .compose([ModelPart::new(
            &source,
            Skinned {
                joints: JointMap::new([(source_joint, joint)]).unwrap(),
                transform: Mat4::IDENTITY,
            },
        )])
        .unwrap();
    let hook = composed
        .features
        .hooks()
        .iter()
        .find(|(hook, _)| hook.id == feature)
        .unwrap()
        .0;
    let index = composed
        .nodes
        .iter()
        .position(|node| node.id == hook.node)
        .unwrap();
    assert_eq!(
        composed.nodes[index].rest.pivot,
        Vec3::from_array([0., 0., 2.])
    );
    assert!(local_matrix(composed.nodes[index].rest).is_finite());
    let empty = HashMap::new();
    let pose = PoseContext {
        nodes: &composed.nodes,
        transition: None,
        overrides: &empty,
        camera: None,
    };
    let sample = composed
        .animation
        .sample(AnimationSample {
            clip: Some(clip),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let sampled = pose
        .sample_feature_node(
            index,
            &sample.at_times(Time::seconds(0.).unwrap(), Time::seconds(0.).unwrap()),
            &mut Vec::new(),
        )
        .0;
    assert!(sampled
        .transform_point3(Vec3::ZERO)
        .abs_diff_eq(Vec3::new(3., 0., 2.), 1e-5));
}

#[test]
fn repeated_rigid_parts_share_geometry_and_skinned_subassemblies_retain_mounts() {
    let mut slot_values = SlotValues::default();
    let (host, joint, clip) = model(1.);
    let (source, source_joint, _) = model(3.);
    let (gem, _, _) = model(0.5);
    let host = compile_model(&host, |_| None).unwrap();
    let source = compile_model(&source, |_| None).unwrap();
    let gem = compile_model(&gem, |_| None).unwrap();
    let source = source
        .compose([ModelPart::new(
            &gem,
            Mounted(ModelMount::new(MountTarget::Node(source_joint))),
        )])
        .unwrap();
    let combined = host
        .compose([
            ModelPart::new(
                &source,
                Skinned {
                    joints: JointMap::new([(source_joint, joint)]).unwrap(),
                    transform: Mat4::IDENTITY,
                },
            ),
            ModelPart::new(&gem, Mounted(ModelMount::new(MountTarget::Node(joint)))),
            ModelPart::new(
                &gem,
                Mounted(ModelMount {
                    target: MountTarget::Node(joint),
                    transform: ModelTransform {
                        translation: Vec3::Y,
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    ..ModelMount::new(MountTarget::Node(joint))
                }),
            ),
        ])
        .unwrap();
    assert_eq!(combined.mounts.len(), 3);
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = combined.prepare(&mut meshes, &mut binds);
    assert_eq!(
        meshes.len(),
        3,
        "repeated rigid source meshes must remain shared"
    );
    let playback = Playback::for_clip(clip);
    let baked = PoseBaker::default()
        .bake(
            &prepared,
            &meshes,
            &binds,
            PoseOptions {
                playback: Some(&playback),
                ..SlotConstruct::construct(&mut slot_values)
            },
        )
        .unwrap();
    assert_eq!(baked.parts.len(), 5);
    let VertexAttributeValues::Float32x3(gem_positions) = baked.parts[2]
        .mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
    else {
        panic!("gem positions")
    };
    assert!(Vec3::from_array(gem_positions[0]).abs_diff_eq(Vec3::new(3.5, 0., 0.), 1e-5));
    assert!(baked.parts.iter().all(|part| part.bounds.is_some()));
}

#[test]
fn mounted_baking_accepts_independent_clock_snapshots() {
    let mut slot_values = SlotValues::default();
    let (host, joint, clip) = model(1.);
    let (part, _, part_clip) = model(3.);
    let host = compile_model(&host, |_| None).unwrap();
    let part = compile_model(&part, |_| None).unwrap();
    let composed = host
        .compose([ModelPart::new(
            &part,
            Mounted(ModelMount::new(MountTarget::Node(joint))),
        )])
        .unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = composed.prepare(&mut meshes, &mut binds);
    let host_playback = Playback::for_clip(clip);
    let mut child_playback = Playback::for_clip(part_clip);
    child_playback.clip_time = 0.5;
    let clocks = HashMap::from([(vec![0], child_playback)]);
    let pose = PoseBaker::default()
        .bake(
            &prepared,
            &meshes,
            &binds,
            PoseOptions {
                playback: Some(&host_playback),
                mounted_playback: Some(&clocks),
                space: BakeSpace::Authored,
                ..SlotConstruct::construct(&mut slot_values)
            },
        )
        .unwrap();
    let VertexAttributeValues::Float32x3(positions) = pose.parts[1]
        .mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
    else {
        panic!("positions")
    };
    assert!(Vec3::from_array(positions[1]).abs_diff_eq(
        Vec3::new(
            4. + 0.5 / std::f32::consts::SQRT_2,
            0.5 / std::f32::consts::SQRT_2,
            0.
        ),
        1e-5
    ));
}

#[test]
fn attachment_mounts_retain_activation_when_baking() {
    let mut slot_values = SlotValues::default();
    use wc3::model::{Attachment as ModelAttachment, NodeFeature, Socket, SocketIdentity};
    let (mut body, joint, clip) = model(1.);
    let point = body
        .edit(|builder| {
            let edit_result = builder.add_feature(
                joint,
                NodeFeature::Attachment(ModelAttachment::Socket(Socket {
                    identity: SocketIdentity::Role(6),
                    visibility: slot_values.add(0.),
                    control: None,
                })),
            );
            builder.import_values(&slot_values)?;
            edit_result
        })
        .unwrap();
    let (armor, _, _) = model(3.);
    let body = compile_model(&body, |_| None).unwrap();
    let armor = compile_model(&armor, |_| None).unwrap();
    let composed = body
        .compose([ModelPart::new(
            &armor,
            Mounted(ModelMount::new(MountTarget::Socket(point))),
        )])
        .unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = composed.prepare(&mut meshes, &mut binds);
    assert_eq!(prepared.mounts().next().unwrap().socket, Some(point));
    let playback = Playback::for_clip(clip);
    let baked = PoseBaker::default()
        .bake(
            &prepared,
            &meshes,
            &binds,
            PoseOptions {
                playback: Some(&playback),
                ..SlotConstruct::construct(&mut slot_values)
            },
        )
        .unwrap();
    assert_eq!(baked.parts.len(), 2);
    assert!(baked.parts[0].visible);
    assert!(!baked.parts[1].visible);
}

#[test]
fn resource_only_placement_is_not_an_attachment_point() {
    use wc3::model::{Attachment, ChildModel, NodeFeature, PlacementAppearance};
    let (mut body, joint, _) = model(1.);
    let placement = body
        .edit(|builder| {
            let resource = builder.add_model_resource(wc3::model::ResourceLocator::Path(
                b"child.mdx".as_slice().into(),
            ));
            builder.add_feature(
                joint,
                NodeFeature::Attachment(Attachment::Child(ChildModel {
                    resource,
                    socket: None,
                    appearance: PlacementAppearance::Unmodified,
                })),
            )
        })
        .unwrap();
    let body = compile_model(&body, |_| None).unwrap();
    let (part, _, _) = model(3.);
    let part = compile_model(&part, |_| None).unwrap();
    assert!(body
        .compose([ModelPart::new(
            &part,
            Mounted(ModelMount::new(MountTarget::Socket(placement)))
        )])
        .is_err());
    assert!(body
        .compose([ModelPart::new(
            &part,
            Mounted(ModelMount::new(MountTarget::Node(joint)))
        )])
        .is_ok());
}

#[test]
fn physical_placements_preserve_explicit_child_frame() {
    let mut slot_values = SlotValues::default();
    use wc3::model::{Axis, Basis, CoordinatePolicy, Handedness};
    let (body, joint, clip) = model(1.);
    let (part, _, _) = model(3.);
    let mut body = compile_model(&body, |_| None).unwrap();
    let mut part = compile_model(&part, |_| None).unwrap();
    // Explicit compiled-frame controls isolate composition from native model metadata.
    let target = Basis::new(Axis::PositiveY, Axis::NegativeZ, Handedness::Right).unwrap();
    body.basis = Transform::from_matrix(
        Basis::default().conversion_to(target) * Mat4::from_scale(Vec3::splat(0.01)),
    );
    part.basis = Transform::from_scale(Vec3::splat(2.));
    part.up = Vec3::Y;
    let offset = ModelTransform {
        translation: Vec3::from_array([2., 3., 4.]),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let physical = body
        .compose([ModelPart::new(
            &part,
            Mounted(ModelMount {
                transform: offset,
                coordinates: CoordinatePolicy::Child,
                ..ModelMount::new(MountTarget::Node(joint))
            }),
        )])
        .unwrap();
    // A child unit spans two meters, or 200 host centimeters.
    // Child Y-up becomes host Z-up, independently of the placement offset.
    assert!(position(&physical, clip, 0., 1).abs_diff_eq(Vec3::new(603., 3., 4.), 1e-3));
    assert!(physical.mounts[0]
        .transform
        .to_matrix()
        .transform_point3(Vec3::Y)
        .abs_diff_eq(Vec3::new(2., 3., 204.), 1e-3));
    let authored = body
        .compose([ModelPart::new(
            &part,
            Mounted(ModelMount {
                transform: offset,
                ..ModelMount::new(MountTarget::Node(joint))
            }),
        )])
        .unwrap();
    assert!(position(&authored, clip, 0., 1).abs_diff_eq(Vec3::new(6., 3., 4.), 1e-5));
    let mut invalid = ModelMount::new(MountTarget::Node(joint));
    invalid.transform.rotation = Quat::from_array([0.; 4]);
    assert!(body
        .compose([ModelPart::new(&part, Mounted(invalid))])
        .is_err());
}
