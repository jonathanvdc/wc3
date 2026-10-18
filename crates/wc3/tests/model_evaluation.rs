//! Offline consumers use semantic values throughout; no image or engine assets are needed.
use glam::{Affine3A, EulerRot, Mat4, Quat, Vec2, Vec3};
use std::{borrow::Cow, collections::HashMap};
use wc3::model::conversion::ImportOptions;
use wc3::model::io::mdlx;
use wc3::model::profiles::mdlx::Material as MdlxMaterial;
use wc3::model::profiles::Universal;
use wc3::model::ModelBuilder;
use wc3::model::ModelMetadata;
use wc3::model::{
    AnimationSample, Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Basis, Curve,
    DeformableGeometry, DynamicTrack, GeometryData, LengthUnit, Mesh, MeshAppearance, Model,
    ModelPose, PoseInputs, PoseLayout, PrimitiveKind, Time, TimeSource, Track, Transform,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

fn time(seconds: f64) -> Time {
    Time::seconds(seconds).unwrap()
}
fn building() -> Model<'static> {
    mdlx::decode_mdl(
        include_str!("fixtures/mdl/quad_model.mdl"),
        &ImportOptions::default(),
    )
    .unwrap()
    .value
    .widen()
}

#[test]
fn standing_snapshot_retains_bindings_and_supports_editable_geometry_without_images() {
    let mut slot_values = SlotValues::default();
    let mut model = building();
    let root = model.nodes().iter().next().unwrap().0;
    model
        .edit(|draft| {
            draft.add_global_track(DynamicTrack::Vector3(Track {
                target: draft.model().node(root).unwrap().transform.translation,
                clock: TimeSource::Animation,
                curve: Curve::linear([time(0.), time(1.)], [[0.; 3], [0., 0., 4.]])?,
            }))
        })
        .unwrap();
    let layout = model.prepare_pose().unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            clip_time: time(0.5),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let pose = layout.evaluate(&sample, PoseInputs::default()).unwrap();
    let appearances = MeshAppearance::compile(&model);
    let frame = model
        .coordinate_transform(Basis::default(), LengthUnit::Unspecified)
        .unwrap();
    let mut joints = Vec::new();
    let mut matrices = Vec::new();
    let mut skinned_parts = 0;
    for (instance_id, instance) in model.instances() {
        let mesh = model.mesh(instance.mesh).unwrap();
        for draw in &instance.draws {
            let material_id = model
                .material_for_draw(instance_id, draw.id(), None)
                .unwrap();
            let material = model.material(material_id).unwrap();
            let surface = MdlxMaterial::try_from(material.surface.clone()).unwrap();
            let frozen = sample.evaluate();
            let texture = frozen[surface.steps[0].shader.inputs().binding.image].unwrap();
            assert!(model.image_resource(texture.resource).is_some());
            assert_eq!(
                sample.animated(&surface.steps[0].shader.inputs().binding.image),
                frozen[surface.steps[0].shader.inputs().binding.image]
            );
            let tint = appearances
                .get(&instance.mesh)
                .map_or([1.; 4], |v| v.sample(&sample));
            assert_eq!(tint, [1.; 4]);
            for mut geometry in mesh
                .project(mesh.subset(draw.subset).unwrap(), 0, [None; 3])
                .unwrap()
            {
                assert_eq!(geometry.kind, PrimitiveKind::Triangles);
                let uv_before = geometry.texture_coordinates.clone();
                if let Some(skin_id) = instance.skin.or(mesh.skin()) {
                    skinned_parts += 1;
                    let skin = model.skin(skin_id).unwrap();
                    let weights = geometry.skin_weights(mesh, &skin.joints).unwrap();
                    layout
                        .joint_matrices_into(skin, &pose, &weights.palette, frame, &mut joints)
                        .unwrap();
                    weights.matrices_into(&joints, &mut matrices).unwrap();
                } else {
                    matrices.resize(
                        geometry.positions.len(),
                        frame * pose.nodes[layout.index(instance.node).unwrap()].matrix,
                    );
                }
                geometry.deform(&matrices).unwrap();
                assert!(geometry.positions.iter().all(|p| (p[2] - 2.).abs() < 1e-6));
                assert_eq!(geometry.texture_coordinates, uv_before);
                assert_eq!(geometry.bounds().unwrap().unwrap().minimum[2], 2.);
                // Application normalization and subsequent encoding need no renderer conversion.
                for p in &mut geometry.positions {
                    p[2] -= 2.;
                }
                assert_eq!(geometry.bounds().unwrap().unwrap().maximum[2], 0.);
            }
        }
    }
    assert!(
        skinned_parts > 0,
        "the standing fixture must exercise authored skinning"
    );
}

#[test]
fn projection_carries_application_face_and_corner_fields_across_triangulation() {
    let mut mesh: Mesh<'static> = Mesh::new(
        GeometryData::triangles(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
            vec![[0, 1, 2], [2, 1, 3]],
        )
        .unwrap(),
    );
    let field = AttributeSemantic::Named(Cow::Borrowed("application-region"));
    mesh.geometry
        .attributes
        .insert(Attribute {
            semantic: field.clone(),
            domain: AttributeDomain::Face,
            values: AttributeValues::Unsigned([17, 23].into_iter().collect()),
        })
        .unwrap();
    let subset = mesh.whole_geometry().unwrap();
    let part = mesh.project(&subset, 0, [None; 3]).unwrap().remove(0);
    let Some(AttributeValues::Unsigned(values)) = part.attribute(&mesh, &subset, &field).unwrap()
    else {
        panic!("projected region")
    };
    assert_eq!(values.iter().collect::<Vec<_>>(), [17, 17, 17, 23, 23, 23]);
    assert_eq!(
        part.sources.iter().map(|c| c.vertex).collect::<Vec<_>>(),
        [0, 1, 2, 2, 1, 3]
    );
}

#[test]
fn repeated_pose_sampling_reuses_buffers_and_retimed_sampling_preserves_overrides() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let root = builder.add_named_node("root").unwrap();
    let definition = builder.new_node("child").with_parent(root);
    let child = builder.add_node(definition).unwrap();
    builder
        .add_global_track(DynamicTrack::Vector3(Track {
            target: builder.model().node(root).unwrap().transform.translation,
            clock: TimeSource::Animation,
            curve: Curve::linear([time(0.), time(1.)], [[0.; 3], [4., 0., 0.]]).unwrap(),
        }))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let layout = model.prepare_pose().unwrap();
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    let mut output = ModelPose::default();
    layout
        .evaluate_into(&sample, PoseInputs::default(), &mut output)
        .unwrap();
    let pointers = (output.nodes.as_ptr(), output.visible.as_ptr());
    let retimed = sample.at_times(time(0.5), time(0.5));
    layout
        .evaluate_into(&retimed, PoseInputs::default(), &mut output)
        .unwrap();
    assert_eq!(pointers, (output.nodes.as_ptr(), output.visible.as_ptr()));
    assert_eq!(
        output.nodes[layout.index(child).unwrap()].matrix.w_axis.x,
        2.
    );
    let overrides = HashMap::from([(
        root,
        Transform {
            translation: [7., 0., 0.],
            ..SlotConstruct::construct(&mut slot_values)
        },
    )]);
    layout
        .evaluate_into(
            &retimed,
            PoseInputs {
                overrides: Some(&overrides),
                ..SlotConstruct::construct(&mut slot_values)
            },
            &mut output,
        )
        .unwrap();
    assert_eq!(
        output.nodes[layout.index(child).unwrap()].matrix.w_axis.x,
        7.
    );
    assert!(layout
        .evaluate(
            &sample,
            PoseInputs {
                camera: Some(Affine3A::from_scale(Vec3::ZERO)),
                ..SlotConstruct::construct(&mut slot_values)
            }
        )
        .is_err());
    let mut malformed = layout.nodes().to_vec();
    malformed[0].parent = Some(1);
    assert!(PoseLayout::new(malformed).is_err());
}

#[test]
fn deformation_preserves_normal_covectors_and_reflected_tangent_handedness() {
    let mut positions = [[1., 2., 3.]];
    let mut normals = [[1., 1., 0.]];
    let mut tangents = [[1., 0., 0., 1.]];
    let matrix =
        Mat4::from_scale_rotation_translation(Vec3::new(-2., 3., 4.), Quat::IDENTITY, Vec3::Y);
    DeformableGeometry {
        positions: &mut positions,
        normals: &mut normals,
        tangents: &mut tangents,
    }
    .deform(&[matrix])
    .unwrap();
    assert_eq!(positions[0], [-2., 7., 12.]);
    assert!(
        Vec3::from_array(normals[0]).abs_diff_eq(Vec3::new(-0.5, 1. / 3., 0.).normalize(), 1e-6)
    );
    assert_eq!(tangents[0], [-1., 0., 0., -1.]);
    let original = positions;
    assert!(DeformableGeometry {
        positions: &mut positions,
        normals: &mut normals,
        tangents: &mut tangents
    }
    .deform(&[])
    .is_err());
    assert_eq!(positions, original);
}

#[test]
fn cpu_skinning_keeps_more_than_eight_influences() {
    use wc3::model::{GeometryInfluences, Influence, ValueGroups};
    let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let joints: Vec<_> = (0..9)
        .map(|i| builder.add_named_node(format!("joint {i}")).unwrap())
        .collect();
    let mut mesh: Mesh<'static> = Mesh::new(
        GeometryData::triangles(
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            vec![[0, 1, 2]],
        )
        .unwrap(),
    );
    mesh.geometry.influences = GeometryInfluences::Variable(ValueGroups::from_rows(
        (0..3).map(|_| (0..9).map(|joint| Influence { joint, weight: 1. })),
    ));
    let subset = mesh.whole_geometry().unwrap();
    let mut part = mesh.project(&subset, 0, [None; 3]).unwrap().remove(0);
    let weights = part.skin_weights(&mesh, &joints).unwrap();
    assert_eq!(weights.palette.len(), 9);
    assert!(weights.rows.iter().all(|row| row.len() == 9));
    let transforms: Vec<_> = (0..9)
        .map(|i| Mat4::from_translation(Vec3::X * i as f32))
        .collect();
    let mut matrices = Vec::new();
    weights.matrices_into(&transforms, &mut matrices).unwrap();
    part.deform(&matrices).unwrap();
    assert!((part.positions[0][0] - 4.).abs() < 1e-5);
    assert!(weights
        .matrices_into(&transforms[..8], &mut matrices)
        .is_err());
}

#[test]
fn sampled_surfaces_freeze_animated_values_and_keep_native_state() {
    let mut slot_values = SlotValues::default();

    use wc3::model::{SurfaceColor, TextureTransform};
    let mut model = building();
    let material_id = model.materials().iter().next().unwrap().0;
    let opacity = MdlxMaterial::try_from(model.material(material_id).unwrap().surface.clone())
        .unwrap()
        .steps[0]
        .shader
        .inputs()
        .opacity;
    let selection = MdlxMaterial::try_from(model.material(material_id).unwrap().surface.clone())
        .unwrap()
        .steps[0]
        .shader
        .inputs()
        .binding
        .image;
    let texture = model.sampled_images()[0];
    let mut material = model.material(material_id).unwrap().clone();
    let mut surface = MdlxMaterial::try_from(material.surface.clone()).unwrap();
    surface.steps[0].shader.inputs_mut().opacity = opacity;
    surface.steps[0].shader.inputs_mut().binding.image = selection;
    material.surface = surface.into();
    model
        .edit(|draft| {
            draft.set_material(material_id, material)?;
            draft.add_global_track(DynamicTrack::Scalar(Track {
                target: opacity,
                clock: TimeSource::Animation,
                curve: Curve::linear([time(0.), time(1.)], [0.2, 0.8])?,
            }))?;
            draft.add_global_track(DynamicTrack::OptionalTexture(Track {
                target: selection,
                clock: TimeSource::Animation,
                curve: Curve::step([time(0.), time(1.)], [None, Some(texture)])?,
            }))
        })
        .unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            clip_time: time(0.5),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let mut source =
        MdlxMaterial::try_from(model.material(material_id).unwrap().surface.clone()).unwrap();
    source.steps[0].shader.inputs_mut().opacity = opacity;
    source.steps[0].shader.inputs_mut().binding.image = selection;
    let frozen_values = sample.evaluate();
    let frozen = source.clone();
    assert!((frozen_values[frozen.steps[0].shader.inputs().opacity] - 0.5).abs() < 1e-6);
    assert_eq!(
        frozen_values[frozen.steps[0].shader.inputs().binding.image],
        None
    );
    assert_eq!(
        frozen.steps[0].shader.inputs().opacity,
        source.steps[0].shader.inputs().opacity
    );
    assert_eq!(frozen.steps[0].state, source.steps[0].state);
    assert_eq!(
        frozen.steps[0].shader.inputs().binding.coordinates,
        source.steps[0].shader.inputs().binding.coordinates
    );
    let color = SurfaceColor {
        opacity: Some(&source.steps[0].shader.inputs().opacity),
        fallback: [0.8, 0.6, 0.4, 1.],
        ..SlotConstruct::construct(&mut slot_values)
    };
    assert_eq!(
        color.sample(&sample, [0.5, 1., 1., 0.5]),
        [0.4, 0.6, 0.4, 0.25]
    );
    let transform = TextureTransform {
        scale: slot_values.add([2., 2., 1.]),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let origin = Vec2::ZERO;
    assert_eq!(
        transform
            .affine(&slot_values, false)
            .transform_point2(origin),
        origin
    );
    assert_eq!(
        transform
            .affine(&slot_values, true)
            .transform_point2(origin),
        Vec2::splat(-0.5)
    );
}

#[test]
fn native_controls_are_not_an_evaluation_input() {
    use wc3::model::controls::model::BoneControls;
    use wc3::model::{Billboard, BillboardAxis, NodeControls, Quaternion};
    let native = mdlx::decode_mdl(
        "Version { FormatVersion 800, } Model \"explicit\" {} Helper \"parent\" { ObjectId 0,  } Helper \"child\" { ObjectId 1, Parent 0, BillboardedLockX, DontInheritTranslation, DontInheritRotation, } PivotPoints 2 { { 1, 2, 3 }, { -1, 2, 1 }, }",
        &ImportOptions::default(),
    ).unwrap().value;
    let mut model: Model = native.widen();
    let camera = Some(Affine3A::from_rotation_translation(
        Quat::from_euler(EulerRot::XYZ, 0.3, -0.7, 0.8),
        Vec3::new(4., 6., 8.),
    ));
    let parent = model
        .nodes()
        .iter()
        .find(|(_, node)| node.name == "parent")
        .unwrap()
        .1
        .transform;
    let mut values = model.values().clone();
    values.set(parent.translation, [2., 3., 4.]).unwrap();
    values
        .set(
            parent.rotation,
            Quaternion(Quat::from_rotation_z(0.4).to_array()),
        )
        .unwrap();
    values.set(parent.scale, [2., -3., 1.]).unwrap();
    let layout = model.prepare_pose().unwrap();
    let expected = layout
        .evaluate(
            &values,
            PoseInputs {
                camera,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(layout
        .nodes()
        .iter()
        .any(|node| node.billboard == Billboard::Axial(BillboardAxis::X)));
    for controls in [None, Some(NodeControls::M2(BoneControls::default()))] {
        let nodes: Vec<_> = model
            .nodes()
            .iter()
            .map(|(id, node)| {
                let mut node = node.clone();
                node.controls = controls.clone();
                (id, node)
            })
            .collect();
        model
            .edit(|draft| {
                for (id, node) in nodes {
                    draft.set_node(id, node)?;
                }
                Ok(())
            })
            .unwrap();
        let actual = model
            .prepare_pose()
            .unwrap()
            .evaluate(
                &values,
                PoseInputs {
                    camera,
                    ..Default::default()
                },
            )
            .unwrap();
        for (actual, expected) in actual.nodes.iter().zip(&expected.nodes) {
            assert_eq!(actual.matrix, expected.matrix);
            assert_eq!(actual.rotation, expected.rotation);
            assert_eq!(actual.scale, expected.scale);
        }
    }
}

#[test]
fn explicit_spherical_modes_preserve_pivots_and_selected_scale_measure() {
    use wc3::model::{
        Billboard, BillboardFrame, BillboardScale, EvaluatedPose, Inheritance, PoseBehavior,
        Quaternion, TransformSemantics,
    };
    let camera =
        Affine3A::from_rotation_translation(Quat::from_rotation_y(0.8), Vec3::new(7., -4., 3.));
    let pose = Transform {
        translation: [1., 2., 3.],
        pivot: [2., -1., 3.],
        rotation: Quaternion(Quat::from_rotation_z(0.4).to_array()),
        scale: [-1., 2., 0.],
    };
    for semantics in [TransformSemantics::Affine, TransformSemantics::PivotLocal] {
        for frame in [BillboardFrame::Camera, BillboardFrame::Yz] {
            for scale in [BillboardScale::SignedFactors, BillboardScale::BasisLengths] {
                for inherit_rotation in [true, false] {
                    let behavior = PoseBehavior {
                        inheritance: Inheritance {
                            rotation: inherit_rotation,
                            ..Default::default()
                        },
                        transform_semantics: semantics,
                        billboard: Billboard::Spherical { frame, scale },
                        ..Default::default()
                    };
                    let parent = EvaluatedPose {
                        matrix: Mat4::from_scale_rotation_translation(
                            Vec3::new(2., 3., -1.),
                            Quat::from_rotation_x(0.3),
                            Vec3::new(4., 1., -2.),
                        ),
                        rotation: Quat::from_rotation_x(0.3),
                        scale: Vec3::new(2., 3., -1.),
                        ..Default::default()
                    };
                    let before = behavior.evaluate(pose, parent, None);
                    let after = behavior.evaluate(pose, parent, Some(camera));
                    assert!(after.matrix.is_finite());
                    let pivot = Vec3::from_array(pose.pivot);
                    assert!(after
                        .matrix
                        .transform_point3(pivot)
                        .abs_diff_eq(before.matrix.transform_point3(pivot), 1e-5));
                    match scale {
                        BillboardScale::SignedFactors => assert_eq!(after.scale, before.scale),
                        BillboardScale::BasisLengths => {
                            for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
                                assert!(
                                    (after.matrix.transform_vector3(axis).length()
                                        - before.matrix.transform_vector3(axis).length())
                                    .abs()
                                        < 1e-5
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn pivot_local_inheritance_can_compensate_an_affine_parents_motion() {
    use wc3::model::{EvaluatedPose, Inheritance, PoseBehavior, TransformSemantics};
    let parent = PoseBehavior::default().evaluate(
        Transform {
            translation: [5., 0., 0.],
            ..Default::default()
        },
        EvaluatedPose::default(),
        None,
    );
    let child = PoseBehavior {
        transform_semantics: TransformSemantics::PivotLocal,
        inheritance: Inheritance {
            translation: false,
            ..Default::default()
        },
        ..Default::default()
    }
    .evaluate(
        Transform {
            translation: [2., 0., 0.],
            ..Default::default()
        },
        parent,
        None,
    );
    assert_eq!(
        child.matrix.transform_point3(Vec3::ZERO),
        Vec3::new(2., 0., 0.)
    );
    assert_eq!(child.motion, Vec3::new(2., 0., 0.));
}
