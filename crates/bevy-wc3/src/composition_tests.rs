use super::*;
use crate::{compile_model, BakeSpace, Playback, PoseBaker, PoseOptions};
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use wc3::model::materials::layered::{Layer, LayeredSurface};
use wc3::model::{
    Animated, Channel, Clip, Clock, Curve, Draw, ExplicitBinding, GeometryData, GeometryInfluences,
    Influence, Material, MaterialSurface, Mesh as ModelMesh, MeshInstance, Node as ModelNode,
    Quaternion, Skin, Time, Track, UniversalBinding,
};

fn model(rest: f32) -> (Model<'static>, NodeId, ClipId) {
    let mut builder = Model::<Universal>::builder("bound triangle");
    let joint = builder
        .add_node(
            ModelNode::named("joint")
                .with_transform(ModelTransform {
                    translation: [rest, 0., 0.],
                    ..Default::default()
                })
                .unwrap(),
        )
        .unwrap();
    let skin = builder
        .add_skin(Skin::new(
            vec![joint],
            UniversalBinding::Explicit(
                ExplicitBinding::new([
                    Mat4::from_translation(Vec3::new(-rest, 0., 0.)).to_cols_array_2d()
                ])
                .unwrap(),
            ),
        ))
        .unwrap();
    let mut geometry = GeometryData::triangles(
        [[rest, 0., 0.], [rest + 0.5, 0., 0.], [rest, 0.5, 0.]],
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
            surface: MaterialSurface::Layered(Box::new(LayeredSurface {
                layers: vec![Layer {
                    opacity: Animated::constant(1.),
                    ..Default::default()
                }],
                ..Default::default()
            })),
        })
        .unwrap();
    let mut instance = MeshInstance::new(joint, mesh);
    instance.skin = Some(skin);
    instance.draws.push(Draw::new(subset, Default::default()));
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    let clip = builder
        .add_clip(Clip::named("turn").with_duration(Time::seconds(1.).unwrap()))
        .unwrap();
    builder
        .add_channel(
            clip,
            Channel::Rotation(Track {
                target: joint.rotation(),
                curve: Curve::linear(
                    [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                    [
                        Quaternion::IDENTITY,
                        Quaternion(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array()),
                    ],
                )
                .unwrap(),
                clock: Clock::Clip,
            }),
        )
        .unwrap();
    (builder.finish().unwrap(), joint, clip)
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
    let (body, joint, clip) = model(1.);
    let (armor, _, armor_clip) = model(3.);
    let body = compile_model(&body, |_| None).unwrap();
    let armor = compile_model(&armor, |_| None).unwrap();
    let mut part = ModelPart::new(
        &armor,
        Rigid {
            parent: joint,
            transform: Mat4::from_translation(Vec3::Y),
        },
    );
    part.clips.insert(clip, armor_clip);
    let composed = body.compose([part]).unwrap();
    assert!(position(&composed, clip, 0., 1).abs_diff_eq(Vec3::new(4., 1., 0.), 1e-5));
    assert!(position(&composed, clip, 0.5, 1).is_finite());
}

#[test]
fn mapped_attachment_anchor_retains_source_pivot_and_rest_space() {
    use crate::pose::{local_matrix, PoseContext};
    use wc3::model::{
        AnimationSample, Attachment, AttachmentActivation, AttachmentIdentity, AttachmentPoint,
        NodeFeature,
    };
    let (host, joint, clip) = model(1.);
    let (mut source, source_joint, _) = model(3.);
    let mut source_node = source.node(source_joint).unwrap().clone();
    source_node.transform.pivot = [0., 0., 2.];
    let feature = source
        .edit(|builder| {
            builder.set_node(source_joint, source_node)?;
            builder.add_feature(
                source_joint,
                NodeFeature::Attachment(Attachment::Point(AttachmentPoint {
                    identity: AttachmentIdentity::Role(11),
                    activation: AttachmentActivation::Uncontrolled,
                    control: None,
                })),
            )
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
    assert_eq!(composed.nodes[index].rest.pivot, [0., 0., 2.]);
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
            ..Default::default()
        })
        .unwrap();
    let sampled = pose
        .sample_feature_node(
            index,
            &sample.at_times(
                Time::seconds(0.).unwrap(),
                Time::seconds(0.).unwrap(),
                Time::seconds(0.).unwrap(),
            ),
            &mut Vec::new(),
        )
        .0;
    assert!(sampled
        .transform_point3(Vec3::ZERO)
        .abs_diff_eq(Vec3::new(3., 0., 2.), 1e-5));
}

#[test]
fn repeated_rigid_parts_share_geometry_and_skinned_subassemblies_retain_mounts() {
    let (host, joint, clip) = model(1.);
    let (source, source_joint, _) = model(3.);
    let (gem, _, _) = model(0.5);
    let host = compile_model(&host, |_| None).unwrap();
    let source = compile_model(&source, |_| None).unwrap();
    let gem = compile_model(&gem, |_| None).unwrap();
    let source = source
        .compose([ModelPart::new(
            &gem,
            Rigid {
                parent: source_joint,
                transform: Mat4::IDENTITY,
            },
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
            ModelPart::new(
                &gem,
                Rigid {
                    parent: joint,
                    transform: Mat4::IDENTITY,
                },
            ),
            ModelPart::new(
                &gem,
                Rigid {
                    parent: joint,
                    transform: Mat4::from_translation(Vec3::Y),
                },
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
                ..Default::default()
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
    let (host, joint, clip) = model(1.);
    let (part, _, part_clip) = model(3.);
    let host = compile_model(&host, |_| None).unwrap();
    let part = compile_model(&part, |_| None).unwrap();
    let composed = host
        .compose([ModelPart::new(
            &part,
            Rigid {
                parent: joint,
                transform: Mat4::IDENTITY,
            },
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
                ..Default::default()
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
fn mapped_controller_clips_activate_part_programmes_on_checked_host_timeline() {
    use wc3::model::controls::playback::{
        ControllerSelection as AuthoredControllers, SequenceControls,
    };
    use wc3::model::{ClipControls, Controller};
    let (mut host, joint, host_clip) = model(1.);
    let controls = || {
        Some(ClipControls::M3(SequenceControls {
            timeline_origin: Time::seconds(1.).unwrap(),
            ..Default::default()
        }))
    };
    host.edit(|builder| builder.set_clip_controls(host_clip, controls()))
        .unwrap();
    let (mut source, source_joint, part_clip) = model(3.);
    let (controller, extra) = source
        .edit(|builder| {
            let extra = builder.add_node(ModelNode::named("extra").with_parent(source_joint))?;
            let controller = builder.add_controller(Controller::default())?;
            builder.add_controller_channel(
                controller,
                Channel::Vector3(Track {
                    target: extra.translation(),
                    curve: Curve::linear(
                        [Time::seconds(1.)?, Time::seconds(2.)?],
                        [[0., 0., 0.], [2., 0., 0.]],
                    )?,
                    clock: Clock::Timeline,
                }),
            )?;
            let mut controls = controls();
            if let Some(ClipControls::M3(value)) = &mut controls {
                value.selection = Some(AuthoredControllers {
                    name: "armor".into(),
                    controllers: vec![controller],
                });
            }
            builder.set_clip_controls(part_clip, controls)?;
            Ok((controller, extra))
        })
        .unwrap();
    let host = compile_model(&host, |_| None).unwrap();
    let source = compile_model(&source, |_| None).unwrap();
    let mut part = ModelPart::new(
        &source,
        Skinned {
            joints: JointMap::new([(source_joint, joint)]).unwrap(),
            transform: Mat4::IDENTITY,
        },
    );
    part.clips.insert(host_clip, part_clip);
    let combined = host.compose([part]).unwrap();
    assert!(combined.timing[&host_clip]
        .controllers
        .contains(&controller));
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = combined.prepare(&mut meshes, &mut binds);
    let mut playback = Playback::for_clip(host_clip);
    playback.clip_time = 0.5;
    let sample = prepared
        .animation
        .sample(playback.selection(&prepared).unwrap())
        .unwrap();
    assert_eq!(sample.property(&extra.translation()), Some([1., 0., 0.]));
}

#[test]
fn attachment_mounts_retain_activation_when_baking() {
    use wc3::model::{
        Animated, Attachment as ModelAttachment, AttachmentActivation, AttachmentIdentity,
        AttachmentPoint, NodeFeature,
    };
    let (mut body, joint, clip) = model(1.);
    let point = body
        .edit(|builder| {
            builder.add_feature(
                joint,
                NodeFeature::Attachment(ModelAttachment::Point(AttachmentPoint {
                    identity: AttachmentIdentity::Role(6),
                    activation: AttachmentActivation::Enabled(Animated::constant(false)),
                    control: None,
                })),
            )
        })
        .unwrap();
    let (armor, _, _) = model(3.);
    let body = compile_model(&body, |_| None).unwrap();
    let armor = compile_model(&armor, |_| None).unwrap();
    let composed = body
        .compose([ModelPart::new(
            &armor,
            Attachment {
                point,
                transform: Mat4::IDENTITY,
            },
        )])
        .unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = composed.prepare(&mut meshes, &mut binds);
    assert_eq!(prepared.mounts().next().unwrap().point, Some(point));
    let playback = Playback::for_clip(clip);
    let baked = PoseBaker::default()
        .bake(
            &prepared,
            &meshes,
            &binds,
            PoseOptions {
                playback: Some(&playback),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(baked.parts.len(), 2);
    assert!(baked.parts[0].visible);
    assert!(!baked.parts[1].visible);
}
