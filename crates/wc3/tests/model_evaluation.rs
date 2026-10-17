//! Offline consumers use semantic values throughout; no image or engine assets are needed.
use glam::{Affine3A, Mat4, Quat, Vec2, Vec3};
use std::{borrow::Cow, collections::HashMap};
use wc3::model::conversion::ImportOptions;
use wc3::model::io::mdlx;
use wc3::model::materials::mdlx::LayeredSurface;
use wc3::model::profiles::Universal;
use wc3::model::{
    AnimationSample, Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Basis,
    Channel, Clock, Curve, DeformableGeometry, GeometryData, LengthUnit, Mesh, MeshAppearance,
    Model, ModelPose, Node, PoseInputs, PoseLayout, PrimitiveKind, Time, Track, Transform,
};

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
    let mut model = building();
    let root = model.nodes().iter().next().unwrap().0;
    model
        .edit(|draft| {
            draft.add_timeline_channel(Channel::Vector3(Track {
                target: root.translation(),
                clock: Clock::Timeline,
                curve: Curve::linear([time(0.), time(1.)], [[0.; 3], [0., 0., 4.]])?,
            }))
        })
        .unwrap();
    let layout = model.prepare_pose().unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            timeline_time: time(0.5),
            ..Default::default()
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
            let surface = LayeredSurface::try_from(material.surface.clone()).unwrap();
            let frozen = surface.sampled(&sample);
            let texture = frozen.layers[0].binding.image.default.unwrap();
            assert!(model.image_resource(texture.resource).is_some());
            assert!(frozen.layers[0].binding.image.binding.is_none());
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
    let mut builder = Model::builder("hierarchy");
    let root = builder.add_node(Node::named("root")).unwrap();
    let child = builder
        .add_node(Node::named("child").with_parent(root))
        .unwrap();
    builder
        .add_timeline_channel(Channel::Vector3(Track {
            target: root.translation(),
            clock: Clock::Timeline,
            curve: Curve::linear([time(0.), time(1.)], [[0.; 3], [4., 0., 0.]]).unwrap(),
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let layout = model.prepare_pose().unwrap();
    let sample = model.animation_sample(AnimationSample::default()).unwrap();
    let mut output = ModelPose::default();
    layout
        .evaluate_into(&sample, PoseInputs::default(), &mut output)
        .unwrap();
    let pointers = (output.nodes.as_ptr(), output.visible.as_ptr());
    let retimed = sample.at_times(time(0.5), time(0.5), time(0.5));
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
            ..Default::default()
        },
    )]);
    layout
        .evaluate_into(
            &retimed,
            PoseInputs {
                overrides: Some(&overrides),
                ..Default::default()
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
                ..Default::default()
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
    let mut builder = Model::<Universal>::builder("many joints");
    let joints: Vec<_> = (0..9)
        .map(|i| builder.add_node(Node::named(format!("joint {i}"))).unwrap())
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
    use wc3::model::profiles::mdlx::properties::Layer;
    use wc3::model::{Animated, SurfaceColor, TextureTransform};
    let mut model = building();
    let material_id = model.materials().iter().next().unwrap().0;
    let opacity = Layer(0).opacity().bind(material_id).widen();
    let selection = Layer(0).texture().bind(material_id).widen();
    let texture = model.sampled_images()[0];
    let mut material = model.material(material_id).unwrap().clone();
    let mut surface = LayeredSurface::try_from(material.surface.clone()).unwrap();
    surface.layers[0].opacity = Animated::new(1., opacity.clone());
    surface.layers[0].binding.image = Animated::new(Some(texture), selection.clone());
    material.surface = surface.try_into().unwrap();
    model
        .edit(|draft| {
            draft.set_material(material_id, material)?;
            draft.add_timeline_channel(Channel::Scalar(Track {
                target: opacity.clone(),
                clock: Clock::Timeline,
                curve: Curve::linear([time(0.), time(1.)], [0.2, 0.8])?,
            }))?;
            draft.add_timeline_channel(Channel::OptionalTexture(Track {
                target: selection.clone(),
                clock: Clock::Timeline,
                curve: Curve::step([time(0.), time(1.)], [None, Some(texture)])?,
            }))
        })
        .unwrap();
    let sample = model
        .animation_sample(AnimationSample {
            timeline_time: time(0.5),
            ..Default::default()
        })
        .unwrap();
    let mut source =
        LayeredSurface::try_from(model.material(material_id).unwrap().surface.clone()).unwrap();
    source.layers[0].opacity = Animated::new(1., opacity);
    source.layers[0].binding.image = Animated::new(Some(texture), selection);
    let frozen = source.sampled(&sample);
    assert!((frozen.layers[0].opacity.default - 0.5).abs() < 1e-6);
    assert_eq!(frozen.layers[0].binding.image.default, None);
    assert!(frozen.layers[0].opacity.binding.is_none());
    assert_eq!(frozen.layers[0].state, source.layers[0].state);
    assert_eq!(
        frozen.layers[0].binding.coordinates,
        source.layers[0].binding.coordinates
    );
    let color = SurfaceColor {
        opacity: Some(&source.layers[0].opacity),
        fallback: [0.8, 0.6, 0.4, 1.],
        ..Default::default()
    };
    assert_eq!(
        color.sample(&sample, [0.5, 1., 1., 0.5]),
        [0.4, 0.6, 0.4, 0.25]
    );
    let transform = TextureTransform {
        scale: Animated::constant([2., 2., 1.]),
        ..Default::default()
    };
    let origin = Vec2::ZERO;
    assert_eq!(
        transform.affine(&sample, false).transform_point2(origin),
        origin
    );
    assert_eq!(
        transform.affine(&sample, true).transform_point2(origin),
        Vec2::splat(-0.5)
    );
}
