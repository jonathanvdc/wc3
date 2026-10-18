use super::*;
use std::f32::consts::FRAC_PI_2;
use wc3::model::common::IntoZeroVec as _;
use wc3::model::conversion::{ApproximationPolicy, ExportOptions, ImportOptions};
use wc3::model::formats::m3::{self, Bundle, Document, Md34};
use wc3::model::io;
use wc3::model::materials::m3::MaterialStep;
use wc3::model::profiles::m3::BoneControls;
use wc3::model::profiles::{Universal, M3};
use wc3::model::Quaternion;
use wc3::model::{Channel, Curve, M3Geometry, Time as ModelTime, TimeSource, Track};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

#[test]
fn native_zero_flag_hierarchy_preserves_skinned_rest_and_animated_parent_placement() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::<M3>::builder("parent-local skin");
    let parent_matrix = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 3., 4.),
        Quat::from_rotation_z(FRAC_PI_2),
        Vec3::new(3., 4., 5.),
    );
    let child_matrix = Mat4::from_scale_rotation_translation(
        Vec3::new(0.5, 1., 2.),
        Quat::IDENTITY,
        Vec3::new(1., 2., 3.),
    );
    let parent = builder
        .add_node(
            Node::named("parent", &mut slot_values)
                .with_controls(BoneControls::construct(&mut slot_values))
                .with_transform(
                    ModelTransform {
                        translation: [3., 4., 5.],
                        rotation: Quaternion(Quat::from_rotation_z(FRAC_PI_2).to_array()),
                        scale: [2., 3., 4.],
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    &mut slot_values,
                )
                .unwrap(),
        )
        .unwrap();
    let child = builder
        .add_node(
            Node::named("child", &mut slot_values)
                .with_controls(BoneControls::construct(&mut slot_values))
                .with_parent(parent)
                .with_transform(
                    ModelTransform {
                        translation: [1., 2., 3.],
                        scale: [0.5, 1., 2.],
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    &mut slot_values,
                )
                .unwrap(),
        )
        .unwrap();
    let skin = builder
        .add_skin(Skin::new(
            vec![parent, child],
            ExplicitBinding::new(vec![
                parent_matrix.inverse().to_cols_array_2d(),
                (parent_matrix * child_matrix).inverse().to_cols_array_2d(),
            ])
            .unwrap(),
        ))
        .unwrap();
    let points = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let weights = [[
        Influence {
            joint: 1,
            weight: 1.,
        },
        Influence {
            joint: 0,
            weight: 0.,
        },
        Influence {
            joint: 0,
            weight: 0.,
        },
        Influence {
            joint: 0,
            weight: 0.,
        },
    ]; 3];
    let geometry = M3Geometry::new(
        points.to_vec().into_zero_vec(),
        vec![[0., 0., 1.]; 3].into_zero_vec(),
        vec![[0, 1, 2]].into_zero_vec(),
        weights.to_vec().into_zero_vec(),
    )
    .unwrap()
    .with_texture_coordinates(vec![vec![[0., 0.], [1., 0.], [0., 1.]].into_zero_vec()])
    .unwrap();
    let mut mesh = ModelMesh::new(geometry);
    let subset = mesh.add_primary_subset(0, 1);
    let mesh = builder.add_mesh(mesh).unwrap();
    let material = builder
        .add_material(SemanticMaterial {
            name: "plain".into(),
            surface: MaterialStep::Standard(Box::new(SlotConstruct::construct(&mut slot_values))),
        })
        .unwrap();
    let frame = builder.add_named_node("").unwrap();
    let mut instance = MeshInstance::new(frame, mesh);
    instance.skin = Some(skin);
    instance.draws.push(Draw::new(subset, None));
    builder
        .add_instance_with_materials(instance, vec![material])
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let options = ExportOptions {
        approximation: ApproximationPolicy::Allow { tolerance: 0.004 },
        ..SlotConstruct::construct(&mut slot_values)
    };
    let encoded = io::m3::plan::<m3::StarCraft2>(&model, &options)
        .unwrap()
        .execute()
        .unwrap()
        .value
        .write()
        .unwrap();
    let bundle = Bundle::from_model(Document::<Md34>::read(&encoded.model).unwrap());
    let mut source = io::m3::import(&bundle, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let parent = source
        .nodes()
        .iter()
        .find(|(_, node)| node.name == "parent")
        .unwrap()
        .0;
    source
        .edit(|builder| {
            builder.add_global_channel(Channel::Vector3(Track {
                target: builder.model().node(parent).unwrap().transform.translation,
                clock: TimeSource::Animation,
                curve: Curve::linear(
                    [ModelTime::seconds(0.)?, ModelTime::seconds(1.)?],
                    [[3., 4., 5.], [5., 4., 5.]],
                )?,
            }))
        })
        .unwrap();
    let compiled = compile_model(&source, |_| None).unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    for (time, displacement) in [(0., 0.), (0.5, 1.), (1., 2.)] {
        let playback = Playback {
            clip_time: time,
            ..SlotConstruct::construct(&mut slot_values)
        };
        let pose = PoseBaker::default()
            .bake(
                &prepared,
                &meshes,
                &binds,
                PoseOptions {
                    playback: Some(&playback),
                    space: BakeSpace::Authored,
                    ..SlotConstruct::construct(&mut slot_values)
                },
            )
            .unwrap();
        let Some(VertexAttributeValues::Float32x3(positions)) =
            pose.parts[0].mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("baked positions missing")
        };
        for (actual, expected) in positions.iter().zip(points) {
            assert!(
                Vec3::from_array(*actual)
                    .abs_diff_eq(Vec3::from_array(expected) + Vec3::X * displacement, 1e-4),
                "time {time}: {actual:?}"
            );
        }
    }
}
